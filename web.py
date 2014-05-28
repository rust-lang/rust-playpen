#!/usr/bin/env python3

import functools
import os
import subprocess
import sys
from bottle import get, post, request, run, static_file

@get("/")
def serve_index():
    return static_file("web.html", root="static")

@get("/<path:path>")
def serve_static(path):
    return static_file(path, root="static")

@functools.lru_cache(maxsize=256)
def playpen(version, command, arguments):
    print("running:", version, command, arguments, file=sys.stderr, flush=True)
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

@post("/evaluate.json")
def evaluate():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    optimize = request.json["optimize"]
    if optimize not in ("0", "1", "2", "3"):
        return {"error": "invalid optimization level"}
    (out, _) = playpen(version, "/usr/local/bin/evaluate.sh", (optimize, request.json["code"]))
    return {"result": out}

@post("/format.json")
def format():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    (out, rc) = playpen(version, "/usr/local/bin/format.sh", (request.json["code"],))
    if rc:
        return {"error": out}
    else:
        return {"result": out[:-1]}

@post("/compile.json")
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
    (out, rc) = playpen(version, "/usr/local/bin/compile.sh", (optimize, emit, request.json["code"]))
    if rc:
        return {"error": out}
    else:
        return {"result": out}

os.chdir(sys.path[0])
run(host='0.0.0.0', port=80, server='cherrypy')
