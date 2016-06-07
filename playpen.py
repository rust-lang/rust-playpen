#!/usr/bin/env python3

import subprocess

def execute(version, command, arguments, data=None):
    with subprocess.Popen(("docker",
                           "run",
                           "--rm",
                           "--cap-drop=ALL",
                           "--memory=128m",
                           "-i",
                           "rust-" + version,
                           command) + arguments,
                           stdin=subprocess.PIPE,
                           stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT) as p:
        if data is None:
            out = p.communicate()[0]
        else:
            out = p.communicate(data.encode())[0]
        return (out, p.returncode)
