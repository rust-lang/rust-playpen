#!/usr/bin/env python3

import subprocess

def execute(version, command, arguments):
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
        return (p.communicate()[0].decode(), p.returncode)
