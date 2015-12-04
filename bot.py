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

def bitly(command):
    bitly = "https://api-ssl.bitly.com/v3/shorten"
    server = "https://play.rust-lang.org/?"

    params = urlencode({"code": command, "run": 1})
    url = server + params

    r = requests.get(bitly,
                     params={"access_token": shorten_key.bitly, "longUrl": url})
    response = r.json()

    if response["status_txt"] == "OK":
        return "output truncated; full output at: " + response["data"]["url"]
    else:
        print(response)
        return "failed to shorten url"

def evaluate(code, channel, template):
    if "%(version)s" in template and "%(input)s" in template:
        version, _ = playpen.execute(channel, "/bin/dash",
                                     ("-c", "--", "rustc -V | head -1 | tr -d '\n'"))
        code = template % {
                "version": version.decode(),
                "input": code }

    out, _ = playpen.execute(channel, "/usr/local/bin/evaluate.sh",
                             ("-C","opt-level=2",), code)

    if len(out.strip().replace(b"\xff", b"", 1)) == 0:
        return "<success, but no output>"

    if len(out) > 5000:
        return "more than 5000 bytes of output; bailing out"

    out = out.replace(b"\xff", b"", 1).decode(errors="replace")
    lines = out.splitlines()

    for line in lines:
        if len(line) > 150:
            return bitly(code)

    limit = 3
    if len(lines) > limit:
        return "\n".join(lines[:limit - 1] + [bitly(code)])

    return out

class RustEvalbot(irc.client.SimpleIRCClient):
    def __init__(self, nickname, channels, keys, password, triggers, default_template):
        irc.client.SimpleIRCClient.__init__(self)
        irc.client.ServerConnection.buffer_class = irc.buffer.LenientDecodingLineBuffer
        self.nickname = nickname
        self.channels = channels
        self.keys = keys
        for t in triggers:
            t['triggers'] = [re.compile(s) for s in t['triggers']]
        self.triggers = triggers
        self.default_template = default_template
        self.password = password

    def _run(self, irc_channel, code, rust_channel, with_template):
        result = evaluate(code, rust_channel, with_template)
        for line in result.splitlines():
            self.connection.notice(irc_channel, line)

    def on_welcome(self, connection, event):
        if self.password:
            connection.privmsg('NickServ', 'identify ' + self.password)
        for channel, key in zip(self.channels, self.keys):
            if key is None:
                connection.join(channel)
            else:
                connection.join(channel, key)

    def on_pubmsg(self, connection, event):
        msg = event.arguments[0]
        for t in self.triggers:
            for r in t['triggers']:
                res = r.match(msg)
                if res:
                    code = res.group(1)
                    self.handle_pubmsg(event, code, t['channel'], t['template'])
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
        # Allow for the same triggers like in channels,
        # but fallback to stable with template.
        for t in self.triggers:
            for r in t['triggers']:
                res = r.match(msg)
                if res:
                    code = res.group(1)
                    self._run(nickname, code, t['channel'], t['template'])
                    # Only one match per message
                    return

        self._run(nickname, msg, "stable", self.default_template)

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

def start(nickname, server, port, channels, keys, password, triggers, default_template):
    client = RustEvalbot(nickname, channels, keys, password, triggers, default_template)
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

    thread = threading.Thread(target=start, args=(cfg["nickname"],
                                                  cfg["server"],
                                                  cfg["port"],
                                                  cfg["channels"],
                                                  cfg["keys"],
                                                  cfg["password"],
                                                  cfg["triggers"],
                                                  cfg["default_template"]))
    thread.start()

if __name__ == "__main__":
    main()
