#!/usr/bin/env python3

import os
import re
import itertools
import subprocess
import sys
import threading
from time import sleep
from urllib.parse import urlencode, quote

import irc.client
import requests
import yaml

import playpen
import shorten_key

irc_template = """\
#![allow(dead_code, unused_variables)]

static VERSION: &'static str = "%(version)s";

fn show<T: std::fmt::Debug>(e: T) { println!("{:?}", e) }

fn main() {
    show({
        %(input)s
    });
}"""

def bitly(command):
    bitly = "https://api-ssl.bitly.com/v3/shorten"
    server = "http://play.rust-lang.org/?"

    params = urlencode({"code": command, "run": 1})
    url = server + params

    r = requests.get(bitly,
                     params={"access_token": shorten_key.key, "longUrl": url})
    response = r.json()

    if response["status_txt"] == "OK":
        return "output truncated; full output at: " + response["data"]["url"]
    else:
        print(response)
        return "failed to shorten url"

def gist(version, code):
    url = "https://api.github.com/gists"

    r = requests.post("https://api.github.com/gists",
            json = {
                "description": "Shared via Rust Playground",
                "public": True,
                "files": {
                    "playbot.rs": {
                        "content": code
                    }
                }
            })

    if r.status_code == 201:
        response = r.json()

        gist_id = response["id"]
        gist_url = response["html_url"]

        play_url = "https://play.rust-lang.org/?gist=" + quote(gist_id) + "&version=" + version
        return "output truncated; full output at: " + play_url
    else:
        return "failed to shorten url"

def pastebin(channel, code):
    if channel == "nightly":
        return gist(channel, code)
    else:
        return bitly(code)

def evaluate(code, channel, with_template):
    if with_template:
        version, _ = playpen.execute(channel, "/bin/dash",
                                     ("-c", "--", "rustc -V | head -1 | tr -d '\n'"))
        code = irc_template % {
                "version": version.decode(),
                "input": code }

    out, _ = playpen.execute(channel, "/usr/local/bin/evaluate.sh",
                             ("-C","opt-level=2",), code)

    if len(out) > 5000:
        return "more than 5000 bytes of output; bailing out"

    out = out.replace(b"\xff", b"", 1).decode(errors="replace")
    lines = out.splitlines()

    for line in lines:
        if len(line) > 150:
            pastebin(channel, code)

    limit = 3
    if len(lines) > limit:
        return "\n".join(lines[:limit - 1] + [pastebin(channel, code)])

    return out

class RustEvalbot(irc.client.SimpleIRCClient):
    def __init__(self, nickname, channels, keys, triggers):
        irc.client.SimpleIRCClient.__init__(self)
        irc.client.ServerConnection.buffer_class = irc.buffer.LenientDecodingLineBuffer
        self.nickname = nickname
        self.channels = channels
        self.keys = keys
        self.triggers = [(re.compile(r), c, wt) for (r, c, wt) in triggers]

    def _run(self, irc_channel, code, rust_channel, with_template):
        result = evaluate(code, rust_channel, with_template)
        for line in result.splitlines():
            self.connection.notice(irc_channel, line)

    def on_welcome(self, connection, event):
        for channel, key in zip(self.channels, self.keys):
            if key is None:
                connection.join(channel)
            else:
                connection.join(channel, key)

    def on_pubmsg(self, connection, event):
        msg = event.arguments[0]
        for (trigger, channel, with_template) in self.triggers:
            res = trigger.match(msg)
            if res:
                code = res.group(1)
                self.handle_pubmsg(event, code, channel, with_template)
                # Only one match per message
                return

    def handle_pubmsg(self, event, code, channel, with_template):
        nickname = event.source.split("!")[0]
        print("{} {}: {}".format(event.target, self.nickname, code))
        self._run(event.target, code, channel, with_template)

    def on_privmsg(self, connection, event):
        nickname = event.source.split("!")[0]
        msg = event.arguments[0]
        print("{} {}: {}".format(event.target, nickname, msg))
        self._run(nickname, msg, "nightly", True)

    def on_disconnect(self, connection, event):
        sleep(10)
        connection.reconnect()

    def on_kick(self, connection, event):
        channel = event.target
        key = self.keys[self.channels.index(channel)]
        if key is None:
            connection.join(channel)
        else:
            connection.join(channel, key)

def start(nickname, server, port, channels, keys, triggers):
    client = RustEvalbot(nickname, channels, keys, triggers)
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

    cfg = cfg[0]
    thread = threading.Thread(target=start, args=(cfg["nickname"],
                                                  cfg["server"],
                                                  cfg["port"],
                                                  cfg["channels"],
                                                  cfg["keys"],
                                                  cfg["triggers"]))
    thread.start()

if __name__ == "__main__":
    main()
