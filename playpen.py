#!/usr/bin/env python3

import subprocess
import re, os.path

g_wrapper_cmd_cache = {}

def execute(channel, command, arguments, data=None, skip_playpen=False, debug_me=False):
    cache_key = (channel, command)
    wrapper_cmd = g_wrapper_cmd_cache.get(cache_key, None)
    if wrapper_cmd is None:
        if skip_playpen:
            wrapper_cmd = ()
            if os.path.exists(command):
                with open(command) as fp:
                    if re.match(r'#!.*sh', fp.readline()):
                        wrapper_cmd = ("sh", )  # compile.sh has non-standard #!/usr/bin/dash
        else:
            wrapper_cmd = ("playpen",
                           "root-" + channel,
                           "--mount-proc",
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=/dev/urandom:r,/dev/null:rw",
                           "--memory-limit=128",
                           "--")
        g_wrapper_cmd_cache[cache_key] = wrapper_cmd

    full_cmd = wrapper_cmd + (command, ) + arguments
    if debug_me:
        print("running {}".format(full_cmd), file=sys.stderr, flush=True)

    with subprocess.Popen(full_cmd,
                          stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT) as p:
        if data is None:
            out = p.communicate()[0]
        else:
            out = p.communicate(data.encode())[0]
        return (out, p.returncode)
