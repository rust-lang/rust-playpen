#!/usr/bin/env python3

import functools
import os
import sys
import getopt
import datetime

from bottle import get, request, response, route, run, static_file
from pygments import highlight
from pygments.formatters import HtmlFormatter
from pygments.lexers import GasLexer, LlvmLexer
from wsgiref.handlers import format_date_time

import playpen

g_debug_me = False
g_skip_playpen = False

CHANNELS = ("stable", "beta", "nightly")
BOOTTIME = int((datetime.datetime.utcnow() - datetime.datetime.utcfromtimestamp(0)).
    total_seconds())

# read-only after initialization by init_rustc_version_json_cache()
g_rustc_version_json_cache = {}

@get("/")
def serve_index():
    response = static_file("web.html", root="static")

    # XSS protection is a misfeature unleashed upon the world by Internet
    # Explorer 8. It uses ill conceived heuristics to block or mangle HTTP
    # requests in an attempt to prevent cross-site scripting exploits. It's yet
    # another idea from the "enumerating badness" school of security theater.
    #
    # Rust and JavaScript are both languages using a C style syntax, and GET
    # queries containing Rust snippets end up being classified as cross-site
    # scripting attacks. Luckily, there's a header for turning off this bug.
    response.set_header("X-XSS-Protection", "0")

    return response

@get("/<path:path>")
def serve_static(path):
    return static_file(path, root="static")

@functools.lru_cache(maxsize=256)
def execute(channel, command, arguments, code):
    print("running:", channel, command, arguments, file=sys.stderr, flush=True)
    return playpen.execute(channel, command, arguments, code, g_skip_playpen, g_debug_me)

def enable_post_cors(wrappee):
    def wrapper(*args, **kwargs):
        response.headers["Access-Control-Allow-Origin"] = "*"
        response.headers["Access-Control-Allow-Methods"] = "POST, OPTIONS"
        response.headers["Access-Control-Allow-Headers"] = "Origin, Accept, Content-Type"

        if request.method != "OPTIONS":
            return wrappee(*args, **kwargs)

    return wrapper

def extractor(key, default, valid):
    def decorator(wrappee):
        def wrapper(*args, **kwargs):
            value = request.json.get(key, default)
            if value not in valid:
                return {"error": "invalid value for {}".format(key)}
            return wrappee(value, *args, **kwargs)
        return wrapper
    return decorator

def init_rustc_version_json_cache():  # called only once in main()
    for channel in CHANNELS:
        try:
            rustc_ver, _ = execute(channel, "rustc", ("--version", ), code=None)
            rustc_ver = str(rustc_ver, "utf-8").strip()
        except FileNotFoundError:
            rustc_ver = "Unknown"
        try:
            rustfmt_ver, _ = execute(channel, "rustfmt", ("--version", ), code=None)
            rustfmt_ver = str(rustfmt_ver, "utf-8").strip()
        except FileNotFoundError:
            rustfmt_ver = "Unknown"
        g_rustc_version_json_cache[channel] = \
            { "rustc": rustc_ver, "rustfmt": rustfmt_ver }

@route("/version.json")
def evaluate():
    response.set_header("Last-Modified", format_date_time(BOOTTIME))
    return g_rustc_version_json_cache

@route("/evaluate.json", method=["POST", "OPTIONS"])
@enable_post_cors
@extractor("color", False, (True, False))
@extractor("test", False, (True, False))
@extractor("version", CHANNELS[0], CHANNELS)
@extractor("optimize", "2", ("0", "1", "2", "3"))
def evaluate(optimize, channel, test, color):
    args = []

    # --evaluatesh=* passed to evaluate.sh itself must precede rustc options
    if g_debug_me:
        args.append("--evaluatesh=debug")
    if test and color:
        args.append("--evaluatesh=coloredtest")

    args.extend(["-C", "opt-level=" + optimize])
    if optimize == "0":
        args.append("-g")
    if color:
        args.append("--color=always")
    if test:
        args.append("--test")

    out, _ = execute(channel, "bin/evaluate.sh", tuple(args), request.json["code"])

    if request.json.get("separate_output") is True:
        split = out.split(b"\xff", 1)

        ret = {"rustc": split[0].decode()}
        if len(split) == 2: # compilation succeeded
            ret["program"] = split[1].decode(errors="replace")

        return ret
    else:
        return {"result": out.replace(b"\xff", b"", 1).decode(errors="replace")}

@route("/format.json", method=["POST", "OPTIONS"])
@enable_post_cors
@extractor("version", CHANNELS[0], CHANNELS)
def format(channel):
    out, rc = execute(channel, "rustfmt", (), request.json["code"])
    if rc:
        return {"error": out.decode()}
    else:
        return {"result": out.decode()}

@route("/compile.json", method=["POST", "OPTIONS"])
@enable_post_cors
@extractor("syntax", "att", ("att", "intel"))
@extractor("color", False, (True, False))
@extractor("version", CHANNELS[0], CHANNELS)
@extractor("optimize", "2", ("0", "1", "2", "3"))
@extractor("emit", "asm", ("asm", "llvm-ir", "mir"))
def compile(emit, optimize, channel, color, syntax):
    args = []

    # --compilesh=* passed to compile.sh itself must precede rustc options
    if g_debug_me:
        args.append("--compilesh=debug")

    args.extend(["-C", "opt-level=" + optimize])
    if optimize == "0":
        args.append("-g")
    if color:
        args.append("--color=always")
    if syntax:
        args.append("-C")
        args.append("llvm-args=-x86-asm-syntax=%s" % syntax)
    if emit == "mir":
        args.append("-Zunstable-options")
        args.append("--unpretty=mir")
    else:
        args.append("--emit=" + emit)
    args.append("--crate-type=lib")
    out, _ = execute(channel, "bin/compile.sh", tuple(args), request.json["code"])
    split = out.split(b"\xff", 1)
    if len(split) == 2:
        rustc_output = split[0].decode()
        emitted = split[1].decode()
    else:
        rustc_output = split[0].decode()
        emitted = None
    if emitted is None:
        return {"error": rustc_output}
    else:
        # You know, it might be good to include the rustc output in the same
        # way evaluate.json does rather than this different way. Ah well.
        # Compatibility and all that. Do we care? I really don't know!
        if request.json.get("highlight") is not True:
            return {"result": split[1].decode()}
        if emit == "asm":
            return {"result": highlight(split[1].decode(), GasLexer(), HtmlFormatter(nowrap=True))}
        elif emit == "llvm-ir":
            return {"result": highlight(split[1].decode(), LlvmLexer(), HtmlFormatter(nowrap=True))}
        else:
            return {"result": split[1].decode()}

def main(args):
    '''
    -d: Debugging mode: turn on verbose debugging (and RUST_BACKTRACE=1)
        Change the listening port to 8080 from the default 80.
    -S: Skip playpen and direcly run rustc and the output binary.
        To mitigate the risk, accepts only local connections by default.
    -a addr, -p port: Directly configure HTTP listening address (default: 0.0.0.0:80)
    '''
    opts, args = getopt.getopt(args, "a:p:dSh")
    listen_port=0
    listen_addr=None
    for o,a in opts:
        if o == "-a":
            listen_addr = a
        elif o == "-p":
            listen_port = a
        elif o == "-d":
            global g_debug_me
            g_debug_me = True
        elif o == "-S":
            global g_skip_playpen
            g_skip_playpen = True
            listen_addr = "127.0.0.1"
    if listen_port == 0:
        listen_port = 8080 if g_debug_me else 80
    if listen_addr is None:
        listen_addr = "0.0.0.0"

    os.chdir(sys.path[0])
    init_rustc_version_json_cache()
    run(host=listen_addr, port=listen_port, server='cherrypy')


if "__main__" == __name__:
    main(sys.argv[1: ])
