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

function sample(session, result) {
    var request = new XMLHttpRequest();
    var index = Math.floor(Math.random() * samples);
    console.log(index);
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

addEventListener("DOMContentLoaded", function() {
    var evaluate_button = document.getElementById("evaluate");
    var format_button = document.getElementById("format");
    var result = document.getElementById("result");
    var version = document.getElementById("version");
    var editor = ace.edit("editor");
    var session = editor.getSession();

    session.setMode("ace/mode/rust");
    sample(session, result);

    evaluate_button.onclick = function() {
        evaluate(result, session.getValue(), version.options[version.selectedIndex].text);
    };

    format_button.onclick = function() {
        format(result, session, version.options[version.selectedIndex].text);
    };
}, false);
