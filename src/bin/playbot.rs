extern crate rust_playpen;
extern crate irc;
extern crate toml;
extern crate hyper;
extern crate url;
extern crate serde_json as json;

use rust_playpen::ReleaseChannel;

use irc::client::prelude::*;
use url::percent_encoding::*;
use hyper::client::Client;

use std::fs::{self, File};
use std::io::{self, Read};
use std::iter;
use std::str;
use std::error::Error;

static DEFAULT_CHANNEL: ReleaseChannel = ReleaseChannel::Stable;
static TRIGGERS: &'static [&'static str] = &[
    ">>",
    "playbot-rs ",
    "playbot-rs,",
    "playbot-rs:",
];

struct Playbot {
    conn: IrcServer,
    rust_versions: Vec<String>,
    shorten_key: String,
}

impl Playbot {
    fn new(s: IrcServer) -> Self {
        let mut versions = Vec::new();
        // Note: Keep these in the same order as their discriminant values
        for channel in &[ReleaseChannel::Stable,
                         ReleaseChannel::Beta,
                         ReleaseChannel::Nightly] {
            let (status, output) = rust_playpen::exec(*channel,
                                                      "rustc",
                                                      &[String::from("-V")],
                                                      String::new()).unwrap();
            assert!(status.success(), "couldn't get version (this currently needs to run as root)");
            let version = str::from_utf8(&output).unwrap();
            // Strip the trailing newline
            let version = String::from(version.lines().next().unwrap());
            println!("got {:?} Rust version: {}", channel, version);
            versions.push(version);
        }

        // Read the bitly API token
        let mut key = String::new();
        File::open("bitly_key").unwrap().read_to_string(&mut key).unwrap();
        // Allow trailing newline
        let key = String::from(key.lines().next().unwrap());

        Playbot {
            conn: s,
            rust_versions: versions,
            shorten_key: key,
        }
    }

    /// Shortens a playpen URL containing the given code.
    ///
    /// Returns the short URL.
    fn pastebin(&self, code: &str) -> hyper::Result<String> {
        let playpen_url = format!("https://play.rust-lang.org/?run=1&code={}",
            utf8_percent_encode(code, FORM_URLENCODED_ENCODE_SET));
        let client = Client::new();
        let url = format!(
            "https://api-ssl.bitly.com/v3/shorten?access_token={}&longUrl={}",
            utf8_percent_encode(&self.shorten_key, FORM_URLENCODED_ENCODE_SET),
            utf8_percent_encode(&playpen_url, FORM_URLENCODED_ENCODE_SET));
        let mut response = try!(client.get(&url).send());
        let mut body = String::new();
        try!(response.read_to_string(&mut body));
        let value: json::Value = json::from_str(&body).unwrap();
        let obj = value.as_object().unwrap();
        if obj["status_txt"].as_string().unwrap() == "OK" {
            Ok(String::from(value.lookup("data.url").unwrap().as_string().unwrap()))
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                               format!("server responded with: {}", body)).into())
        }
    }

    fn run_code(&mut self,
                full_code: &str,
                channel: ReleaseChannel)
                -> io::Result<String> {
        let (_status, output) = try!(rust_playpen::exec(channel,
                                                        "/usr/local/bin/evaluate.sh",
                                                        &[],
                                                        String::from(full_code)));

        let mut split = output.splitn(2, |b| *b == b'\xff');
        let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

        match split.next() {
            Some(out) => {
                // Compilation succeeded
                Ok(String::from_utf8_lossy(out).into_owned())
            }
            None => {
                Ok(rustc)
            }
        }
    }

    /// Parse a command sent to playbot (playbot's name needs to be stripped beforehand)
    ///
    /// Returns the response to send to the user (each line is a NOTICE)
    fn parse_and_run(&mut self, msg: &str) -> io::Result<String> {
        // Initialize default attributes:
        let mut channel = DEFAULT_CHANNEL;
        let mut mini = false;   // don't use a template

        // Parse attributes. Attributes are identifiers in front of the code, prefixed with '~'
        let mut start = 0;
        let msg = msg.trim();
        for attr_end in msg.match_indices(char::is_whitespace)
                                .map(|(start, _)| start)
                                .chain(iter::once(msg.len())) {
            let attr = &msg[start..attr_end];
            if !attr.starts_with('~') { break; }
            start = attr_end + 1;

            match attr[1..].trim() {
                "stable" => channel = ReleaseChannel::Stable,
                "beta" => channel = ReleaseChannel::Beta,
                "nightly" => channel = ReleaseChannel::Nightly,
                "mini" => mini = true,
                "help" => {
                    return Ok(format!("\
syntax: {} [~attribute1] ... [~attributeN] <Rust code to execute>
~stable | ~beta | ~nightly: select the Rust version to use
~mini: don't wrap the code in an `fn main` and print the result
", self.conn.current_nickname()));
                }
                unknown => {
                    return Ok(format!("unknown attribute '{}' (try '~help' for a list)", unknown));
                }
            }
        }

        let code = &msg[start..];
        let code = if mini {
            String::from(code)
        } else {
            format!(r#"
#![allow(dead_code, unused_variables)]

static VERSION: &'static str = "{version}";

fn show<T: std::fmt::Debug>(e: T) {{ println!("{{:?}}", e) }}

fn main() {{
    show({{
        {code}
    }});
}}
"#, version = self.rust_versions[channel as usize], code = code)
        };

        let out = try!(self.run_code(&code, channel));
        if out.len() > 5000 {
            return Ok(String::from("output too long, bailing out :("));
        }

        // Print outputs up to 3 lines in length. Above that, print the first 2 lines followed by a
        // shortened playpen link.
        let lines: Vec<&str> = out.lines().collect();
        if lines.len() <= 3 {
            return Ok(lines.join("\n"));
        }

        // Take the first 2 lines and append the URL
        let mut response = lines[..3].join("\n");
        match self.pastebin(&code) {
            Ok(short_url) => response.push_str(&format!("\n(output truncated; full output at {})",
                                                        short_url)),
            Err(e) => {
                log_error(e);
                response.push_str("\n(output truncated; shortening URL failed)");
            }
        }

        Ok(response)
    }

    fn handle_cmd(&mut self, response_to: &str, msg: &str) {
        match self.parse_and_run(msg) {
            Ok(response) => {
                for line in response.lines() {
                    if !line.is_empty() {
                        if let Err(e) = self.conn.send_notice(response_to, line) {
                            log_error(e);
                        }
                    }
                }
            }
            Err(e) => {
                log_error(e);
            }
        }
    }

    /// Called when any user writes a public message
    fn handle_pubmsg(&mut self, from: &str, chan: &str, mut msg: &str) {
        for trig in TRIGGERS {
            if msg.starts_with(trig) {
                msg = &msg[trig.len()..].trim();

                println!("<{}> {}", from, msg);
                self.handle_cmd(chan, msg);
                return;
            }
        }
    }

    /// Called when receiving a private message from `from` (via `/msg playbot-rs ...`)
    fn handle_privmsg(&mut self, from: &str, msg: &str) {
        println!("(/msg) <{}> {}", from, msg);
        self.handle_cmd(from, msg);
    }

    fn main_loop(&mut self) {
        println!("playbot at your service!");
        let cloned = self.conn.clone();
        for msg in cloned.iter() {
            let msg = match msg {
                Ok(msg) => msg,
                // FIXME I'm not sure when this will be returned and whether `continue` is the right
                // response.
                Err(_) => continue,
            };

            let from = match msg.source_nickname() {
                Some(name) => name,
                None => continue,   // no user attached, so it's not interesting for us
                                    // (probably a server msg)
            };
            match msg.command {
                Command::PRIVMSG(ref to, ref msg) => {
                    // `to` is either "#rust" or "playbot-rs", depending on whether the message was
                    // private or public. Obviously, public messages are transmitted as PRIVMSG
                    // because that makes total sense.
                    if cloned.config().channels.as_ref().unwrap().contains(to) {
                        self.handle_pubmsg(from, to, &msg);
                    } else {
                        self.handle_privmsg(from, &msg);
                    }
                },
                _ => {},
            }
        }
    }
}

/// Log and forget an error
fn log_error<E: Error>(e: E) {
    println!("[ERROR] {}", e);
}

fn main() {
    fs::metadata("whitelist").expect("syscall whitelist file not found");

    // FIXME All these unwraps are pretty bad UX, but they should only panic on misconfiguration
    let mut config = String::new();
    File::open("playbot.toml").unwrap().read_to_string(&mut config).unwrap();
    let toml = toml::Parser::new(&config).parse().unwrap();

    let conf = Config {
        nickname: Some(String::from(toml["nick"].as_str().unwrap())),
        nick_password: toml.get("password").map(|val| String::from(val.as_str().unwrap())),
        server: Some(String::from(toml["server"].as_str().unwrap())),
        channels: Some(toml["channels"].as_slice().unwrap()
            .iter()
            .map(|val| String::from(val.as_str().unwrap()))
            .collect()),
        ..Config::default()
    };

    let server = IrcServer::from_config(conf).unwrap();
    server.identify().unwrap();
    Playbot::new(server).main_loop();
}
