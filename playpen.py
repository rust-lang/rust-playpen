#!/usr/bin/env python3

import subprocess

def execute(version, command, arguments, decode=True):
    with subprocess.Popen(("playpen",
                           "root-" + version,
                           "--mount-proc",
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=/dev/urandom:r",
                           "--memory-limit=128",
                           "--",
                           command) + arguments,
                           stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT) as p:
        out = p.communicate()[0]
        return (out.decode(errors="replace") if decode else out,
                p.returncode)
