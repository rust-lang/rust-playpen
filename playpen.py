#!/usr/bin/env python3

import subprocess

def execute(version, command, arguments, data=None):
    with subprocess.Popen(("playpen",
                           "root-" + version,
                           "--mount-proc",
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=/dev/urandom:r,/dev/null:w",
                           "--memory-limit=128",
                           "--",
                           command) + arguments,
                           stdin=subprocess.PIPE,
                           stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT) as p:
        if data is None:
            out = p.communicate()[0]
        else:
            out = p.communicate(data.encode())[0]
        return (out, p.returncode)
