#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate irc;
extern crate rust_playpen;
extern crate rustc_serialize;
extern crate toml;
extern crate url;

use std::fs::File;
use std::io::{self, Read};
use std::str;
use std::sync::Arc;
use std::thread;
use std::u16;

use hyper::client::Client;
use irc::client::prelude::*;
use rust_playpen::{ReleaseChannel, Cache};
use rustc_serialize::json;
use url::form_urlencoded;

static DEFAULT_CHANNEL: ReleaseChannel = ReleaseChannel::Stable;

fn get_rust_versions(cache: &Cache) -> Vec<String> {
    let mut versions = Vec::new();
    // Note: Keep these in the same order as their discriminant values
    for channel in &[ReleaseChannel::Stable,
                     ReleaseChannel::Beta,
                     ReleaseChannel::Nightly] {
        let (status, output) = cache.exec(*channel,
                                          "rustc",
                                          vec![String::from("-V")],
                                          String::new()).unwrap();
        assert!(status.success(), "couldn't get version (this currently needs to run as root)");
        let version = str::from_utf8(&output).unwrap();
        // Strip the trailing newline
        let version = String::from(version.lines().next().unwrap());
        debug!("got {:?} Rust version: {}", channel, version);
        versions.push(version);
    }

    versions
}

struct Playbot {
    conn: IrcServer,
    rust_versions: Vec<String>,
    shorten_key: String,
    cache: Arc<Cache>,
}

impl Playbot {
    /// Shortens a playpen URL containing the given code.
    ///
    /// Returns the short URL.
    fn pastebin(&self, code: &str) -> hyper::Result<String> {
        let playpen_url = format!("https://play.rust-lang.org/?run=1&code={}",
            form_urlencoded::byte_serialize(code.as_bytes()).collect::<String>());
        let client = Client::new();
        let url = format!(
            "https://api-ssl.bitly.com/v3/shorten?access_token={}&longUrl={}",
            form_urlencoded::byte_serialize(self.shorten_key.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(playpen_url.as_bytes()).collect::<String>());
        let mut response = try!(client.get(&url).send());
        let mut body = String::new();
        try!(response.read_to_string(&mut body));
        let value: json::Json = json::Json::from_str(&body).unwrap();
        let obj = value.as_object().unwrap();
        if obj["status_txt"].as_string().unwrap() == "OK" {
            Ok(String::from(value.find_path(&["data", "url"]).unwrap().as_string().unwrap()))
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                               format!("server responded with: {}", body)).into())
        }
    }

    fn run_code(&mut self,
                full_code: &str,
                channel: ReleaseChannel)
                -> io::Result<String> {
        let (_status, output) = try!(self.cache.exec(channel,
                                                     "/usr/local/bin/evaluate.sh",
                                                     Vec::new(),
                                                     String::from(full_code)));

        let output_merged = output.splitn(2, |b| *b == b'\xff')
                                  .map(|sub| String::from_utf8_lossy(sub).into_owned())
                                  .collect::<String>();
        Ok(output_merged)
    }

    /// Parse a command sent to playbot (playbot's name needs to be stripped beforehand)
    ///
    /// Returns the response to send to the user (each line is a NOTICE)
    fn parse_and_run(&mut self, code: &str) -> io::Result<String> {
        let code = if self.conn.current_nickname().contains("mini") {
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
"#, version = self.rust_versions[DEFAULT_CHANNEL as usize], code = code)
        };

        let out = try!(self.run_code(&code, DEFAULT_CHANNEL));
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
                error!("shortening url failed: {}", e);
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
                            error!("couldn't send response: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }

    /// Called when any user writes a public message
    fn handle_pubmsg(&mut self, from: &str, chan: &str, msg: &str) {
        if msg.starts_with(self.conn.current_nickname()) {
            let msg = &msg[self.conn.current_nickname().len()..];

            if msg.len() <= 2 || !msg.starts_with(&[' ', ',', ':'] as &[char]) {
                return;
            }

            let command = &msg[1..];
            info!("<{}> {}", from, command);
            self.handle_cmd(chan, command);
        }
    }

    /// Called when receiving a private message from `from` (via `/msg playbot-rs ...`)
    fn handle_privmsg(&mut self, from: &str, msg: &str) {
        info!("(/msg) <{}> {}", from, msg);
        self.handle_cmd(from, msg);
    }

    fn main_loop(&mut self) {
        info!("playbot at your service!");
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

fn main() {
    env_logger::init().unwrap();

    let cache = Arc::new(Cache::new());
    let rust_versions = get_rust_versions(&cache);

    // FIXME All these unwraps are pretty bad UX, but they should only panic on misconfiguration
    let mut config = String::new();
    File::open("playbot.toml").unwrap().read_to_string(&mut config).unwrap();
    let toml = toml::Parser::new(&config).parse().unwrap();
    let bitly_key = toml["bitly-key"].as_str().unwrap().to_string();

    let mut threads = Vec::new();
    for server in toml["server"].as_slice().unwrap() {
        let server = server.as_table().unwrap();

        for nick in server["nicks"].as_slice().unwrap() {
            let server_addr = server["server"].as_str().unwrap();
            let conf = Config {
                nickname: Some(String::from(nick.as_str().unwrap())),
                nick_password: server.get("password").map(|val| String::from(val.as_str().unwrap())),
                server: Some(String::from(server_addr)),
                port: server.get("port").map(|val| {
                    let port = val.as_integer().unwrap();
                    assert!(0 < port && port < u16::MAX as i64, "out of range for ports");
                    port as u16
                }),
                channels: Some(server["channels"].as_slice().unwrap()
                    .iter()
                    .map(|val| String::from(val.as_str().unwrap()))
                    .collect()),
                ..Config::default()
            };

            let server = IrcServer::from_config(conf).unwrap();
            server.identify().unwrap();
            let mut bot = Playbot {
                conn: server,
                rust_versions: rust_versions.clone(),
                shorten_key: bitly_key.clone(),
                cache: cache.clone(),
            };
            threads.push(thread::Builder::new()
                                         .name(format!("{}@{}", nick, server_addr))
                                         .spawn(move || {
                bot.main_loop();
            }).unwrap());
        }
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
