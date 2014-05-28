#!/usr/bin/env python3

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

def playpen(version, command, arguments):
    return subprocess.Popen(["playpen",
                             "root-" + version,
                             "--mount-proc",
                             "--user=rust",
                             "--timeout=5",
                             "--syscalls-file=whitelist",
                             "--devices=/dev/urandom:r",
                             "--memory-limit=128",
                             "--",
                             command] + arguments,
                             stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT)

@post("/evaluate.json")
def evaluate():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    optimize = request.json["optimize"]
    if optimize not in ("0", "1", "2", "3"):
        return {"error": "invalid optimization level"}
    print(request.json, file=sys.stderr, flush=True)
    with playpen(version, "/usr/local/bin/evaluate.sh", [optimize, request.json["code"]]) as p:
        return {"result": p.stdout.read().decode()}

@post("/format.json")
def format():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    print(request.json, file=sys.stderr, flush=True)
    with playpen(version, "/usr/local/bin/format.sh", [request.json["code"]]) as p:
        output = p.communicate()[0][:-1].decode()
        if p.returncode:
            return {"error": output}
        else:
            return {"result": output}

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
    print(request.json, file=sys.stderr, flush=True)
    with playpen(version, "/usr/local/bin/compile.sh", [optimize, emit, request.json["code"]]) as p:
        output = p.communicate()[0].decode()
        if p.returncode:
            return {"error": output}
        else:
            return {"result": output}

os.chdir(sys.path[0])
run(host='0.0.0.0', port=80, server='cherrypy')
