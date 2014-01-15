#!/usr/bin/env python3

import subprocess
from bottle import get, post, request, run, static_file

@get("/")
def serve_index():
    return static_file("web.html", root="static")

@get("/<path:path>")
def serve_static(path):
    return static_file(path, root="static")

@post("/evaluate.json")
def evaluate():
    version = request.json["version"]
    if version not in ("master",):
        return {"error": "invalid version"}
    print(request.json)
    with subprocess.Popen(["playpen",
                          "root-" + version,
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=c:1:9",
                           "--memory-limit=128M",
                           "--",
                           "/usr/local/bin/evaluate.sh",
                           request.json["code"]],
                           stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT) as p:
        return {"result": p.stdout.read().decode()}

@post("/format.json")
def format():
    version = request.json["version"]
    if version not in ("master", "0.7"):
        return {"error": "invalid version"}
    print(request.json)
    with subprocess.Popen(["playpen",
                           "root-" + version,
                           "--user=rust",
                           "--timeout=5",
                           "--syscalls-file=whitelist",
                           "--devices=c:1:9",
                           "--memory-limit=128M",
                           "--",
                           "/usr/local/bin/format.sh",
                           request.json["code"]],
                           stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT) as p:
        output = p.communicate()[0][:-1].decode()
        if p.returncode:
            return {"error": output}
        else:
            return {"result": output}

run(host='0.0.0.0', port=8000, server='cherrypy')
