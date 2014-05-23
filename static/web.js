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

function evaluate(result, code, version) {
    send("/evaluate.json", {code: code, version: version}, function(rc, object) {
        if (rc == 200) {
            result.textContent = object["result"];
        } else {
            result.textContent = "connection failure";
        }
    });
}

function compile(emit, result, code, version) {
    send("/compile.json", {code: code, version: version, emit: emit}, function(rc, object) {
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

function set_random_sample(sample, session, result) {
    var index = Math.floor(Math.random() * samples);
    set_sample(sample, session, result, index);
}

addEventListener("DOMContentLoaded", function() {
    var evaluate_button = document.getElementById("evaluate");
    var asm_button = document.getElementById("asm");
    var ir_button = document.getElementById("ir");
    var format_button = document.getElementById("format");
    var result = document.getElementById("result");
    var version = document.getElementById("version");
    var sample = document.getElementById("sample");
    var editor = ace.edit("editor");
    var session = editor.getSession();

    session.setMode("ace/mode/rust");
    set_random_sample(sample, session, result);

    sample.onchange = function() {
        set_sample(sample, session, result, sample.selectedIndex);
    };

    evaluate_button.onclick = function() {
        evaluate(result, session.getValue(), version.options[version.selectedIndex].text);
    };

    asm_button.onclick = function() {
        compile("asm", result, session.getValue(), version.options[version.selectedIndex].text);
    };

    ir_button.onclick = function() {
        compile("ir", result, session.getValue(), version.options[version.selectedIndex].text);
    };

    format_button.onclick = function() {
        format(result, session, version.options[version.selectedIndex].text);
    };
}, false);
