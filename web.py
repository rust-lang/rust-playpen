#!/usr/bin/env python3

import functools
import hashlib
import os
import playpen
import re
import sys
from bottle import abort, get, request, response, route, run, static_file

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

# This is not required because we load the code with Javascript, but better
# claim the route explicitly to avoid confusion.
@get("/c/<identifier>")
def serve_preloaded_index(identifier):
    response = static_file("web.html", root="static")
    response.set_header("X-XSS-Protection", "0")
    return response

@get("/raw/<identifier>")
def get_identifier(identifier):
    filename = cache_get_fullpath(identifier) + ".rs"
    if not os.access (filename, os.R_OK):
        abort(404, "Not Found")

    with open(filename, 'r') as f:
        return f.read()

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

def cache_lookup(wrappee):
    def wrapper(*args, **kwargs):
        identifier = cache_get_id(request)
        filename = cache_get_fullpath(identifier)
        cached = cache_json_content(filename)
        if not cached:
            return wrappee(*args, **kwargs)

        return cached

    return wrapper

def cache_json_content(filename):
    if not os.access (filename, os.R_OK):
        return None

    with open(filename, 'r') as f:
        return {"result": f.read(), "id": os.path.basename(filename)}

def cache_update(identifier, code, output):
    filename = cache_get_fullpath(identifier)
    os.makedirs(name=os.path.dirname(filename), exist_ok=True)
    with open(filename + ".rs", "w") as src:
        src.write(code)
    with open(filename, "w") as out:
        out.write(output)

def cache_get_id(request):
    args = [request.json[k] for k in sorted(request.json.keys())]
    args.append(request.script_name)

    return hashlib.md5(''.join(args).encode()).hexdigest()

def cache_get_fullpath(identifier):
    path = os.path.join(*re.findall (".{4}", identifier))

    return os.path.join("cache", path, identifier)

@route("/evaluate.json", method=["POST", "OPTIONS"])
@enable_post_cors
@cache_lookup
def evaluate():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    optimize = request.json["optimize"]
    if optimize not in ("0", "1", "2", "3"):
        return {"error": "invalid optimization level"}
    out, _ = execute(version, "/usr/local/bin/evaluate.sh", (optimize, request.json["code"]))
    cache_id = cache_get_id(request)
    cache_update(cache_id, request.json["code"], out)

    return {"result": out, "id": cache_id}

@route("/format.json", method=["POST", "OPTIONS"])
@enable_post_cors
@cache_lookup
def format():
    version = request.json["version"]
    if version not in ("master", "0.10"):
        return {"error": "invalid version"}
    out, rc = execute(version, "/usr/local/bin/format.sh", (request.json["code"],))
    if rc:
        return {"error": out}
    else:
        cache_id = cache_get_id(request)
        cache_update(cache_id, request.json["code"], out)
        return {"result": out[:-1], "id": cache_id}

@route("/compile.json", method=["POST", "OPTIONS"])
@enable_post_cors
@cache_lookup
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
    out, rc = execute(version, "/usr/local/bin/compile.sh", (optimize, emit, request.json["code"]))
    if rc:
        return {"error": out}
    else:
        cache_id = cache_get_id(request)
        cache_update(cache_id, request.json["code"], out)

        return {"result": out, "id": cache_id}

os.chdir(sys.path[0])
run(host='0.0.0.0', port=80, server='cherrypy')
