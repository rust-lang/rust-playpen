#[macro_use] extern crate log;
extern crate env_logger;
extern crate reqwest;
extern crate irc;
extern crate rust_playpen;
extern crate toml;

use std::fs::File;
use std::io::{self, Read};
use std::str;
use std::sync::Arc;
use std::thread;
use std::u16;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process;
use std::error::Error;

use reqwest::{Client, StatusCode};
use irc::client::prelude::*;
use rust_playpen::{ReleaseChannel, Cache, PLAYPEN_ENV_VAR_NAME};

static DEFAULT_CHANNEL: ReleaseChannel = ReleaseChannel::Stable;

const ENV: &'static str = "irc";

fn base_env() -> Vec<(String, String)> {
    vec![(PLAYPEN_ENV_VAR_NAME.into(), ENV.into())]
}

fn get_rust_versions(cache: &Cache) -> Vec<String> {
    let mut versions = Vec::new();
    // Note: Keep these in the same order as their discriminant values
    for channel in &[ReleaseChannel::Stable,
                     ReleaseChannel::Beta,
                     ReleaseChannel::Nightly] {
        let (status, output) = cache.exec(*channel,
                                          "rustc",
                                          vec![String::from("-V")],
                                          base_env(),
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
    cache: Arc<Cache>,
}

impl Playbot {
    /// Shortens a playpen URL containing the given code.
    ///
    /// Returns the short URL.
    fn pastebin(&self, data: String) -> Result<String, Box<Error>> {
        let client = Client::new()?;
        let mut response = client.post("https://paste.rs/")?
            .body(data)
            .send()?;
        let mut body = String::new();

        if response.status() == StatusCode::Created {
            response.read_to_string(&mut body)?;
            body.pop(); // the response ends with \n
            Ok(body)
        } else {
            Err(format!("server responded with: {}", body).into())
        }
    }

    fn run_code(&mut self,
                full_code: &str,
                channel: ReleaseChannel)
                -> io::Result<String> {
        let (_status, output) = try!(self.cache.exec(channel,
                                                     "/usr/local/bin/evaluate.sh",
                                                     Vec::new(),
                                                     base_env(),
                                                     String::from(full_code)));

        let output_merged = output.splitn(2, |b| *b == b'\xff')
                                  .map(|sub| String::from_utf8_lossy(sub).into_owned())
                                  .collect::<String>();
        Ok(output_merged)
    }

    /// Parse a command sent to playbot (playbot's name needs to be stripped beforehand)
    ///
    /// Returns the response to send to the user (each line is a NOTICE)
    fn parse_and_run(&mut self, mut code: &str) -> io::Result<String> {
        let mut channel = DEFAULT_CHANNEL;

        if code.starts_with("--") {
            let mut parts = code.splitn(2, ' ');
            match parts.next() {
                Some("--stable") => { channel = ReleaseChannel::Stable },
                Some("--beta") => { channel = ReleaseChannel::Beta },
                Some("--nightly") => { channel = ReleaseChannel::Nightly },
                _ => return Ok(String::from("unrecognized release channel")),
            }
            code = parts.next().unwrap_or("");
        }

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
"#, version = self.rust_versions[channel as usize], code = code)
        };

        let out = try!(self.run_code(&code, channel));
        if out.len() > 3000 {
            return Ok(String::from("output too long, bailing out :("));
        }

        // Print outputs up to 2 lines in length. Above that, print the first line followed by
        // the link to the full output
        let lines: Vec<&str> = out.lines().collect();
        if lines.len() <= 2 {
            return Ok(lines.join("\n"));
        }

        // Take the first line and append the URL
        let response = lines[0];
        Ok(match self.pastebin(format!("{}\n\n~~~~~=====OUTPUT=====~~~~~\n\n{}", code, out)) {
            Ok(short_url) => format!("{}\n(output truncated; full output at {})",
                                                        response, short_url),
            Err(e) => {
                error!("shortening url failed: {}", e);
                format!("{}\n(output truncated; shortening URL failed)", response)
            }
        })
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

            if msg.len() < 2 || !msg.starts_with(&[',', ':'] as &[char]) {
                return;
            }

            let command = msg[1..].trim();
            info!("{}: <{}> {}", chan, from, command);
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
        cloned.for_each_incoming(|msg| {
            let from = match msg.source_nickname() {
                Some(name) => name,
                None => return,   // no user attached, so it's not interesting for us
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
                Command::INVITE(_, ref to) => {
                    if cloned.config().channels.as_ref().unwrap().contains(to) {
                        if let Err(e) = self.conn.send_join(to) {
                            error!("couldn't join {}: {}", to, e);
                        }
                    }
                },
                _ => {},
            }
        }).expect("something went south");
    }
}

fn main() {
    env_logger::init().unwrap();

    let cache = Arc::new(Cache::new());
    let rust_versions = get_rust_versions(&cache);

    // FIXME All these unwraps are pretty bad UX, but they should only panic on misconfiguration
    let mut config = String::new();
    File::open("playbot.toml").unwrap().read_to_string(&mut config).unwrap();
    let toml = config.parse::<toml::Value>().unwrap();

    let mut threads = Vec::new();
    for server in toml["server"].as_array().unwrap() {
        let server = server.as_table().unwrap();

        for nick in server["nicks"].as_array().unwrap() {
            let nick = nick.as_str().unwrap();
            let server_addr = server["server"].as_str().unwrap();
            let conf = Config {
                nickname: Some(String::from(nick)),
                nick_password: server.get("password").map(|val| String::from(val.as_str().unwrap())),
                alt_nicks: Some(vec![format!("{}_", nick), format!("{}__", nick)]),
                should_ghost: Some(true),
                ghost_sequence: Some(vec!["RECOVER".to_string()]),
                server: Some(String::from(server_addr)),
                port: server.get("port").map(|val| {
                    let port = val.as_integer().unwrap();
                    assert!(0 < port && port < u16::MAX as i64, "out of range for ports");
                    port as u16
                }),
                channels: Some(server["channels"].as_array().unwrap()
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
                cache: cache.clone(),
            };
            threads.push(thread::Builder::new()
                                         .name(format!("{}@{}", nick, server_addr))
                                         .spawn(move || {
                if let Err(_) = catch_unwind(AssertUnwindSafe(|| bot.main_loop())) {
                    error!("killing playbot due to previous error");

                    // Abort the whole process, killing the other threads. This should make
                    // debugging easier since the other bots don't keep running.
                    process::exit(101);
                }
            }).unwrap());
        }
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn irc_no_version() {
    drop(env_logger::init());

    let cache = Cache::new();
    let input = r#"fn main() {}"#;

    let (status, out) = cache.exec(ReleaseChannel::Stable, "/usr/local/bin/evaluate.sh",
                                   vec![], base_env(), input.into()).unwrap();

    assert!(status.success());
    assert_eq!(&out, b"\xFF"); // 0xFF is a separator produced by evaluate.sh
}
