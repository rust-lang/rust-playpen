"use strict";

var samples = 2;

function send(path, data, callback) {
    var result = document.getElementById("result");

    result.textContent = "Running...";

    var request = new XMLHttpRequest();
    request.open("POST", path, true);
    request.setRequestHeader("Content-Type", "application/json");
    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            var json;

            try {
                json = JSON.parse(request.response);
            }
            catch (e) {
                console.log ("JSON.parse(): " + e);
            }

            callback(request.status, json);
        }
    }
    request.send(JSON.stringify(data));
}

function evaluate(result, code, version, optimize) {
    send("/evaluate.json", {code: code, version: version, optimize: optimize},
         function(rc, object) {
        if (rc == 200) {
            result.textContent = object["result"];

            var div = document.createElement("div");
            div.className = "message";
            div.textContent = "Program ended.";
            result.appendChild(div);
        } else {
            result.textContent = "connection failure";
        }
    });
}

function compile(emit, result, code, version, optimize) {
    send("/compile.json", {emit: emit, code: code, version: version, optimize: optimize,
                           highlight: true},
         function(rc, object) {
        if (rc == 200) {
            if ("error" in object) {
                result.textContent = object["error"];
            } else {
                result.innerHTML = object["result"];
            }
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

function share(result, version, code) {
    var playurl = "http://play.rust-lang.org?code=" + encodeURIComponent(code);
    if (version != "master") {
        playurl += "&version=" + encodeURIComponent(version);
    }
    if (playurl.length > 5000) {
        result.textContent = "resulting URL above character limit for sharing. " +
            "Length: " + playurl.length + "; Maximum: 5000";
        return;
    }

    var url = "http://is.gd/create.php?format=json&url=" + encodeURIComponent(playurl);

    var request = new XMLHttpRequest();
    request.open("GET", url, true);

    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            if (request.status == 200) {
                setResponse(JSON.parse(request.responseText)['shorturl']);
            } else {
                result.textContent = "connection failure";
            }
        }
    }

    request.send();

    function setResponse(shorturl) {
        while(result.firstChild) {
            result.removeChild(result.firstChild);
        }

        var link = document.createElement("a");
        link.href = link.textContent = shorturl;

        result.textContent = "short url: ";
        result.appendChild(link);
    }
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
    var share_button = document.getElementById("share");
    var result = document.getElementById("result");
    var optimize = document.getElementById("optimize");
    var version = document.getElementById("version");
    var sample = document.getElementById("sample");
    var editor = ace.edit("editor");
    var session = editor.getSession();

    editor.setTheme("ace/theme/github");
    session.setMode("ace/mode/rust");

    var query = get_query_parameters();
    if ("code" in query) {
        session.setValue(query["code"]);
    } else {
        var index = Math.floor(Math.random() * samples);
        set_sample(sample, session, result, index);
    }

    if ("version" in query) {
        version.value = query["version"];
    }

    if (query["run"] === "1") {
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

    share_button.onclick = function() {
        share(result, version.value, session.getValue());
    };
}, false);
