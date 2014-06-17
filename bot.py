#!/usr/bin/env python3

import os
import itertools
import subprocess
import sys
import threading
from time import sleep
from urllib.parse import urlencode

import irc.client
import requests
import yaml

import playpen
import shorten_key

irc_template = """\
#![feature(asm, globs, macro_rules, phase, simd, struct_variant, quad_precision_float)]

extern crate collections;
extern crate native;
extern crate rand;
#[phase(syntax)]
extern crate regex_macros;
extern crate regex;

#[allow(dead_code)]
static version: &'static str = "%(version)s";

#[allow(dead_code)]
fn show<T: std::fmt::Show>(e: T) { println!("{}", e) }

fn main() {
    let r = {
        %(input)s
    };
    println!("{}", r)
}"""

def pastebin(command):
    bitly = "https://api-ssl.bitly.com/v3/shorten"
    server = "http://play.rust-lang.org/?"

    params = urlencode({"code": command, "run": 1})
    url = server + params
    r = requests.get(bitly,
                     params={"access_token": shorten_key.key, "longUrl": url})
    response = r.json()

    if response["status_txt"] == "OK":
        return response["data"]["url"]
    else:
        return "failed to shorten url"

def evaluate(script, arguments, template):
    version, _ = playpen.execute("master",
                                 "/bin/dash",
                                 ("-c", "--", "rustc -v | tail | head -1 | tr -d '\n'"))
    out, _ = playpen.execute("master", script, arguments)
    n = 0

    if len(out) > 5000:
        return "more than 5000 bytes of output, bailing out"

    if out.count("\n") > 3:
        return pastebin(template % {"version": version, "input": arguments[-1]})

    for line in out.splitlines():
        if len(line) > 150:
            return pastebin(template % {"version": version, "input": arguments[-1]})

    return out

class RustEvalbot(irc.client.SimpleIRCClient):
    def __init__(self, nickname, channels, keys):
        irc.client.SimpleIRCClient.__init__(self)
        irc.client.ServerConnection.buffer_class = irc.buffer.LenientDecodingLineBuffer
        self.nickname = nickname
        self.channels = channels
        self.keys = keys

    def _run(self, channel, code):
        if self.nickname == "rusti":
            result = evaluate("/usr/local/bin/irc.sh", (code,), irc_template)
        else:
            result = evaluate("/usr/local/bin/evaluate.sh", ("2", code), "%(input)s")
        for line in result.splitlines():
            self.connection.notice(channel, line)

    def on_welcome(self, connection, event):
        for channel, key in zip(self.channels, self.keys):
            if key is None:
                connection.join(channel)
            else:
                connection.join(channel, key)

    def on_pubmsg(self, connection, event):
        nickname = event.source.split("!")[0]
        msg = event.arguments[0]
        if msg.startswith(self.nickname + ": ") or msg.startswith(self.nickname + ", "):
            print("{} {}: {}".format(event.target, nickname, msg))
            i = len(self.nickname) + 2
            self._run(event.target, msg[i:])

    def on_privmsg(self, connection, event):
        nickname = event.source.split("!")[0]
        msg = event.arguments[0]
        print("{} {}: {}".format(event.target, nickname, msg))
        self._run(nickname, msg)

    def on_disconnect(self, connection, event):
        sleep(10)
        connection.reconnect()

def start(nickname, server, port, channels, keys):
    client = RustEvalbot(nickname, channels, keys)
    try:
        client.connect(server, port, nickname)
        client.connection.set_keepalive(30)
    except irc.client.ServerConnectionError as x:
        print(x)
        sys.exit(1)
    client.start()

def main():
    os.chdir(sys.path[0])

    with open("irc.yaml") as f:
        cfg = yaml.load(f.read())

    for c, nickname in itertools.product(cfg, ("rusti", "rustilite")):
        thread = threading.Thread(target=start, args=(nickname,
                                                      c["server"],
                                                      c["port"],
                                                      c["channels"],
                                                      c["keys"]))
        thread.start()

if __name__ == "__main__":
    main()
