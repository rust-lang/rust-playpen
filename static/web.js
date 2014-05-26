"use strict";

var samples = 2;

function send(path, data, callback) {
    var request = new XMLHttpRequest();
    request.open("POST", path, true);
    request.setRequestHeader("Content-Type", "application/json");
    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            callback(request.status, JSON.parse(request.response));
        }
    }
    request.send(JSON.stringify(data));
}

function evaluate(result, code, version, optimize) {
    send("/evaluate.json", {code: code, version: version, optimize: optimize},
         function(rc, object) {
        if (rc == 200) {
            result.textContent = object["result"];
        } else {
            result.textContent = "connection failure";
        }
    });
}

function compile(emit, result, code, version, optimize) {
    send("/compile.json", {emit: emit, code: code, version: version, optimize: optimize},
         function(rc, object) {
        if (rc == 200) {
            result.textContent = object["result"];
        } else {
            result.textContent = "connection failure";
        }
    });
}

function format(result, session, version) {
    send("/format.json", {code: session.getValue(), version: version}, function(rc, object) {
        if (rc == 200) {
            if ("error" in object) {
                result.textContent = object["error"];
            } else {
                result.textContent = "";
                session.setValue(object["result"]);
            }
        } else {
            result.textContent = "connection failure";
        }
    });
}

function set_sample(sample, session, result, index) {
    var request = new XMLHttpRequest();
    sample.options[index].selected = true;
    request.open("GET", "/sample/" + index + ".rs", true);
    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            if (request.status == 200) {
                session.setValue(request.responseText.slice(0, -1));
            } else {
                result.textContent = "connection failure";
            }
        }
    }
    request.send();
}

function get_query_parameters() {
    var a = window.location.search.substr(1).split('&');
    if (a == "") return {};
    var b = {};
    for (var i = 0; i < a.length; i++) {
        var p = a[i].split('=');
        if (p.length != 2) continue;
        b[p[0]] = decodeURIComponent(p[1].replace(/\+/g, " "));
    }
    return b;
}

addEventListener("DOMContentLoaded", function() {
    var evaluate_button = document.getElementById("evaluate");
    var asm_button = document.getElementById("asm");
    var ir_button = document.getElementById("ir");
    var format_button = document.getElementById("format");
    var result = document.getElementById("result");
    var optimize = document.getElementById("optimize");
    var version = document.getElementById("version");
    var sample = document.getElementById("sample");
    var editor = ace.edit("editor");
    var session = editor.getSession();

    session.setMode("ace/mode/rust");

    var query = get_query_parameters();
    if ("code" in query) {
        session.setValue(query["code"]);
    } else {
        var index = Math.floor(Math.random() * samples);
        set_sample(sample, session, result, index);
    }

    if ("run" in query && query["run"] === "1") {
        evaluate(result, session.getValue(), version.options[version.selectedIndex].text,
                 optimize.options[optimize.selectedIndex].value);
    }

    sample.onchange = function() {
        set_sample(sample, session, result, sample.selectedIndex);
    };

    evaluate_button.onclick = function() {
        evaluate(result, session.getValue(), version.options[version.selectedIndex].text,
                 optimize.options[optimize.selectedIndex].value);
    };

    asm_button.onclick = function() {
        compile("asm", result, session.getValue(), version.options[version.selectedIndex].text,
                 optimize.options[optimize.selectedIndex].value);
    };

    ir_button.onclick = function() {
        compile("ir", result, session.getValue(), version.options[version.selectedIndex].text,
                 optimize.options[optimize.selectedIndex].value);
    };

    format_button.onclick = function() {
        format(result, session, version.options[version.selectedIndex].text);
    };
}, false);
