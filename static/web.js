"use strict";

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

/**
 * Evaluate a block of Rust code.
 *
 * @param {String} code
 * @param {String} version
 * @param {String} optimize
 * @param {Function} fn Callback
 */

function evaluate(code, version, optimize, fn) {
    if ('function' !== typeof fn) {
        throw new Error("Expected a function");
    }

    var obj = {code: code, version: version, optimize: optimize};

    send("/evaluate.json", obj, function(rc, object) {
        if (rc == 200) {
            if ("error" in object) {
                fn({ message: object["error"] }, null);
            } else {
                fn(null, object["result"]);
            }
        } else {
            fn({ message: "connection failure"}, null);
        }
    });
}

/**
 * Compile a piece of Rust code and emit a specific output.
 *
 * @param {String} emit
 * @param {String} code
 * @param {String} version
 * @param {String} optimize
 * @param {Function} fn
 */

function compile(emit, code, version, optimize, fn) {
    var obj = {
        emit: emit,
        code: code,
        version: version,
        optimize: optimize
    };

    send("/compile.json", obj, function(rc, object) {
        if (rc == 200) {
            if ("error" in object) {
                fn({ message: object["error"] }, null);
            } else {
                fn(null, object["result"]);
            }
        } else {
            fn({ message: "connection failure" });
        }
    });
}

/**
 * Format a given block of Rust code.
 *
 * @param {String} code
 * @param {String} version
 * @param {Function} fn
 */

function format(code, version, fn) {
    var obj = {
        code: code,
        version: version
    };

    send("/format.json", obj, function(rc, object) {
        if (rc == 200) {
            if ("error" in object) {
                fn({ message: object["error"] }, null);
            } else {
                fn(null, object["result"]);
            }
        } else {
            fn({ message: "connection failure" }, null);
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

/**
 * Sample Constructor
 */

function Sample(index) {
    if (!(this instanceof Sample)) return new Sample(index);
    this.index = index;
}

/**
 * Retrive the sample according to the passed index.
 *
 * Usage:
 *
 * Sample(0).set(function(err, value) {
 *
 * });
 *
 * @param {Function} fn
 */

Sample.prototype.set = function(fn) {
    var request = new XMLHttpRequest();

    request.open("GET", "/sample/" + this.index + ".rs", true);
    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            if (request.status == 200) {
                fn(null, request.responseText.slice(0, -1));
            } else {
                fn({
                    message: "connection failure"
                }, null);
            }
        }
    }

    request.send();
};

/**
 * Keep track of the current selected dropdown to restrict a single dropdown
 * being active at any given time.
 */

var currentSelection = null;

/**
 * A generic dropdown implementation.
 *
 * @param {Element} el A dom element that is the root of the dropdown.
 */

function Dropdown(el) {
    var self = this;

    this.el = el;
    this.selected = this.el.querySelector("li.default").textContent;
    this.index = (function() {
        // Get all the lis:
        var lis = self.el.querySelector("li");

        for (var i = 0; i < lis.length; i++) {
            var li = lis[i];

            if (li.classList.contains("default")) {
                return i;
            }
        }

        return 0;
    }());

    this.callbacks = [];
}

/**
 * Initialize all the events that need to be attached to the dropdown. This
 * includes all the functionality of the dropdown.
 */

Dropdown.prototype.initialize = function() {
    var self = this;

    this.el.onclick = function() {
        if (this !== currentSelection && currentSelection && currentSelection.classList.contains("open")) {
            currentSelection.classList.toggle("open");
        }

        this.classList.toggle("open");

        if (this.classList.contains("open")) {
            currentSelection = this;
        }
    };

    // Look for the options.
    var options = this.el.querySelectorAll("li");
    for (var j = 0; j < options.length; j++) {
        var option = options[j];
        option.__id = j;

        option.onclick = function() {
            var el   = option.parentNode.parentNode.parentNode.querySelector("span");
            el.textContent = this.textContent;
            self.selected = this.textContent;

            self.el.classList.toggle("open");

            for (var i = 0; i < self.callbacks.length; i++) {
                self.callbacks[i](this.textContent, this.__id);
            }
        };
    }
}

/**
 * Add a callback to the queue. All the callbacks will be fired when the dropdown
 * has a new selection, thus changes.
 */

Dropdown.prototype.change = function(fn) {
    if ('function' !== typeof fn) {
        throw new Error("Expected a function.");
    }

    this.callbacks.push(fn);
    return this;
};

/**
 * Starting point for the application.
 */

;(function(window, document, undefined) {

    document.addEventListener("DOMContentLoaded", function() {
        var evaluate_button = document.getElementById("evaluate");
        var asm_button = document.getElementById("asm");
        var ir_button = document.getElementById("ir");
        var format_button = document.getElementById("format");
        var result = document.getElementById("result");

        // Initialize the ace editor:
        var editor = ace.edit("editor");
        var session = editor.getSession();

        session.setMode("ace/mode/rust");
        editor.resize();
        editor.setTheme("ace/theme/github");

        // Initialize the dropdowns. Each index is the DOM id of the dropdown.
        var dropdowns = ['optimize', 'version', 'sample'];
        var collection = {};

        for (var i = 0; i < dropdowns.length; i++) {
            var dropdown = new Dropdown(document.getElementById(dropdowns[i]));
            dropdown.initialize();

            collection[dropdowns[i]] = dropdown;
        }

        var query = get_query_parameters();

        if ("code" in query) {
            session.setValue(query["code"]);
        } else {
            Sample(collection.sample.index).set(function(err, text) {
                if (err) {

                } else {
                    session.setValue(text);
                }
            });
        }

        if (query["run"] === "1") {
            //evaluate(session.getValue(), collection['version'].selected, collection['optimize'].selected);
        }

        collection.sample.change(function(value, index) {
            Sample(index).set(function(err, text) {
                if (err) {

                } else {
                    session.setValue(text);
                }
            });
        });

        evaluate_button.onclick = function() {
            var optimize = collection.optimize.selected.match(/O([0-3])/)[1];
            evaluate(session.getValue(), collection.version.selected, optimize, function(err, response) {
                if (err) {
                    result.textContent = err.message;
                } else {
                    result.textContent = response;
                }
            });
        };

        asm_button.onclick = function() {
            var optimize = collection.optimize.selected.match(/O([0-3])/)[1];
            compile("asm", session.getValue(), collection.version.selected, optimize, function(err, response) {
                if (err) {
                    result.textContent = err.message;
                } else {
                    result.textContent = response;
                }
            });
        };

        ir_button.onclick = function() {
            var optimize = collection.optimize.selected.match(/O([0-3])/)[1];
            compile("ir", session.getValue(), collection.version.selected, optimize, function(err, response) {
                if (err) {
                    result.textContent = err.message;
                } else {
                    result.textContent = response;
                }
            });
        };

        format_button.onclick = function() {
            format(session.getValue(), collection.version.selected, function(err, text) {
                if (err) {
                    result.textContent = err.message;
                } else {
                    session.setValue(text);
                }
            });
        };
    }, false);
}(window, document));

