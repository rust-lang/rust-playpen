#!/usr/bin/env python3

import functools
import os
import sys

from bottle import get, request, response, route, run, static_file
from pygments import highlight
from pygments.formatters import HtmlFormatter
from pygments.lexers import GasLexer, LlvmLexer

import playpen

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
def execute(version, command, arguments, code):
    print("running:", version, command, arguments, file=sys.stderr, flush=True)
    return playpen.execute(version, command, arguments, code)

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

def buildargs(optimize, color, backtrace):
    args = ["-C", "opt-level=" + optimize]
    if "0" == optimize:
        debug = True
    else:
        debug = False
    if "2" == backtrace:
        if debug:
            backtrace="1"
        else:
            backtrace="0"
    if "1" == backtrace:
        args.insert(0, "--backtrace")
        #XXX: If exists, --backtrace must be the first arg
        #     that is passed to compile.sh and evaluate.sh
    if debug:
        args.append("-g")
    if color:
        args.append("--color=always")
    return args

@route("/evaluate.json", method=["POST", "OPTIONS"])
@enable_post_cors
@extractor("backtrace", "0", ("0", "1", "2"))
@extractor("color", False, (True, False))
@extractor("test", False, (True, False))
@extractor("version", "stable", ("stable", "beta", "nightly"))
@extractor("optimize", "2", ("0", "1", "2", "3"))
def evaluate(optimize, version, test, color, backtrace):
    args = buildargs(optimize, color, backtrace)
    if test:
        args.append("--test")

    out, _ = execute(version, "/usr/local/bin/evaluate.sh", tuple(args), request.json["code"])

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
@extractor("version", "stable", ("stable", "beta", "nightly"))
def format(version):
    out, rc = execute(version, "/usr/bin/rustfmt", (), request.json["code"])
    if rc:
        return {"error": out.decode()}
    else:
        return {"result": out.decode()}

@route("/compile.json", method=["POST", "OPTIONS"])
@enable_post_cors
@extractor("backtrace", "0", ("0", "1", "2"))
@extractor("syntax", "att", ("att", "intel"))
@extractor("color", False, (True, False))
@extractor("version", "stable", ("stable", "beta", "nightly"))
@extractor("optimize", "2", ("0", "1", "2", "3"))
@extractor("emit", "asm", ("asm", "llvm-ir", "mir"))
def compile(emit, optimize, version, color, syntax, backtrace):
    args = buildargs(optimize, color, backtrace)
    if syntax:
        args.append("-C")
        args.append("llvm-args=-x86-asm-syntax=%s" % syntax)
    if emit == "mir":
        args.append("-Zunstable-options")
        args.append("--unpretty=mir")
    else:
        args.append("--emit=" + emit)
    out, _ = execute(version, "/usr/local/bin/compile.sh", tuple(args), request.json["code"])
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

os.chdir(sys.path[0])
run(host='0.0.0.0', port=80, server='cherrypy')
