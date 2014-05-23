#!/usr/bin/env python3

import subprocess
from bottle import get, post, request, run, static_file

@get("/")
def serve_index():
    return static_file("web.html", root="static")

@get("/<path:path>")
def serve_static(path):
    return static_file(path, root="static")

def playpen(version, command, argument):
    return subprocess.Popen(["playpen",
                             "root-" + version,
                             "--mount-proc",
                             "--user=rust",
                             "--timeout=5",
                             "--syscalls-file=whitelist",
                             "--devices=/dev/urandom:r",
                             "--memory-limit=128",
                             "--",
                             command,
                             argument],
                             stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT)

@post("/evaluate.json")
def evaluate():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    print(request.json)
    with playpen(version, "/usr/local/bin/evaluate.sh", request.json["code"]) as p:
        return {"result": p.stdout.read().decode()}

@post("/format.json")
def format():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    print(request.json)
    with playpen(version, "/usr/local/bin/format.sh", request.json["code"]) as p:
        output = p.communicate()[0][:-1].decode()
        if p.returncode:
            return {"error": output}
        else:
            return {"result": output}

run(host='0.0.0.0', port=8000, server='cherrypy')
