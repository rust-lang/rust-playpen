#!/usr/bin/env python3

import functools
import os
import playpen
import sys
from bottle import get, request, response, route, run, static_file

@get("/")
def serve_index():
    return static_file("web.html", root="static")

@get("/<path:path>")
def serve_static(path):
    return static_file(path, root="static")

@functools.lru_cache(maxsize=256)
def execute(version, command, arguments):
    print("running:", version, command, arguments, file=sys.stderr, flush=True)
    return playpen.execute(version, command, arguments)

def enable_post_cors(wrappee):
    def wrapper(*args, **kwargs):
        response.headers["Access-Control-Allow-Origin"] = "*"
        response.headers["Access-Control-Allow-Methods"] = "POST, OPTIONS"
        response.headers["Access-Control-Allow-Headers"] = "Origin, Accept, Content-Type"

        if request.method != "OPTIONS":
            return wrappee(*args, **kwargs)

    return wrapper

@route("/evaluate.json", method=["POST", "OPTIONS"])
@enable_post_cors
def evaluate():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    optimize = request.json["optimize"]
    if optimize not in ("0", "1", "2", "3"):
        return {"error": "invalid optimization level"}
    out, _ = execute(version, "/usr/local/bin/evaluate.sh", (optimize, request.json["code"]))
    return {"result": out}

@route("/format.json", method=["POST", "OPTIONS"])
@enable_post_cors
def format():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    out, rc = execute(version, "/usr/local/bin/format.sh", (request.json["code"],))
    if rc:
        return {"error": out}
    else:
        return {"result": out[:-1]}

@route("/compile.json", method=["POST", "OPTIONS"])
@enable_post_cors
def compile():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    emit = request.json["emit"]
    if emit not in ("asm", "ir"):
        return {"error": "invalid emission type"}
    optimize = request.json["optimize"]
    if optimize not in ("0", "1", "2", "3"):
        return {"error": "invalid optimization level"}
    out, rc = playpen(version, "/usr/local/bin/compile.sh", (optimize, emit, request.json["code"]))
    if rc:
        return {"error": out}
    else:
        return {"result": out}

os.chdir(sys.path[0])
run(host='0.0.0.0', port=80, server='cherrypy')
