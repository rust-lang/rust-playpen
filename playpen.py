#!/usr/bin/env python3

import subprocess
import shlex #for quote() - python3 required
import re #for re.match()

def execute(version, command, arguments, data=None, env_vars=None):
    #Note:
    #arguments is expected to be a tuple!
    #env_vars can be either a list or a tuple

    if env_vars: #this means it's not empty and it's not None
        #if we have env vars, we need wrap everything around the shell in order to be able to set environment variables eg. dash -c 'envvars cmd args'
        #eg: playpen ppargs -- /usr/bin/dash -c 'RUST_BACKTRACE=1 evaluate.sh -C --test --color=always'
        exported_vars = ""
        for env_var in env_vars:
            #eg. RUST_BACKTRACE=1
            var_halves = env_var.split("=", 1)
            var_name = var_halves[0]
            if len(var_halves) == 2:
                var_value = var_halves[1]
            else:
                var_value = None
            if not re.match("^[a-zA-Z0-9_]+$", var_name):
                raise NameError("Bad env.var name, you supplied: \""
                    +env_var+"\"")
            var_rebuilt = var_name
            if None != var_value:
                var_rebuilt += "=" + shlex.quote(var_value)
            exported_vars += "export " + var_rebuilt + "; "
            #exportedvars += varrebuilt + " " #this would be without 'export'
            #XXX: Why 'export' instead of without it? to ensure all cases are
            #covered, for example: /bin/echo and the built-in echo don't work
            #without it eg. (A=1 B=2 echo "$A $B")
            # vs. (export A=1; export B=2; echo "$A $B")
            #And since this is inside playpen, exported vars won't stick around
            #until the next run.

        #the extra space between vars and command already exists from above.
        as_one_arg = exported_vars + command
        for arg in arguments:
            as_one_arg += " " + shlex.quote(arg)
        command = "/usr/bin/dash" #why dash? because evaluate.sh had it set
        arguments = ("-c", as_one_arg)

    #Example with env.vars: ('playpen', 'root-nightly', '--mount-proc', '--user=rust',
    #'--timeout=5', '--syscalls-file=whitelist', '--devices=/dev/urandom:r,/dev/null:rw',
    #'--memory-limit=128', '--', '/usr/bin/dash', '-c',
    #"export RUST_BACKTRACE=1; export T_1000; export something_else='nothing new'; /usr/local/bin/evaluate.sh --backtrace -C opt-level=0 -g --color=always")
    #
    #Example without: ('playpen', 'root-nightly', '--mount-proc', '--user=rust',
    #'--timeout=5', '--syscalls-file=whitelist', '--devices=/dev/urandom:r,/dev/null:rw',
    #'--memory-limit=128', '--', '/usr/local/bin/evaluate.sh',
    #'--backtrace', '-C', 'opt-level=0', '-g', '--color=always')
    with subprocess.Popen(("playpen",
                           "root-" + version,
                           "--mount-proc",
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=/dev/urandom:r,/dev/null:rw",
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
