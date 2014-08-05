;(function(window, document, undefined) {

// ECMAScript 6 Backwards compatability
if (typeof String.prototype.startsWith != 'function') {
  String.prototype.startsWith = function(str) {
    return this.slice(0, str.length) == str;
  };
}

function escapeHTML(unsafe) {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
    .replace(newLineRegex, '<br />');
}
// Regex for finding new lines
var newLineRegex = /(?:\r\n|\r|\n)/g;
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
    this.initialize();
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

        option.onclick = function(event) {
            var el   = option.parentNode.parentNode.parentNode.querySelector("span");
            var target = event.target;

            el.textContent = this.textContent;
            self.selected = this.textContent;

            self.el.classList.toggle("open");

            var lis = self.el.querySelectorAll("li");

            for (var i = 0; i < lis.length; i++) {
                var li = lis[i];

                if (li === target) {
                    self.index = i;
                }
            }

            for (var i = 0; i < self.callbacks.length; i++) {
                self.callbacks[i](this.textContent, this.__id);
            }
        };
    }

    return this;
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

    var obj = {code: code, version: version, optimize: optimize, separate_output: true};

    send("/evaluate.json", obj, function(statusCode, res) {
        if (statusCode == 200) {
            fn(null, res);
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
        optimize: optimize,
        separate_output: true
    };

    send("/compile.json", obj, function(status, res) {
        if (status == 200) {
            fn(null, res);
        } else {
            fn({ message: "connection failure" });
        }
    });
}

function share(version, code, fn) {
    var playurl = "http://play.rust-lang.org?code=" + encodeURIComponent(code);

    if (version != "master") {
        playurl += "&version=" + encodeURIComponent(version);
    }

    if (playurl.length > 5000) {
        return fn("resulting URL above character limit for sharing. " +
            "Length: " + playurl.length + "; Maximum: 5000", null);
    }

    var url = "http://is.gd/create.php?format=json&url=" + encodeURIComponent(playurl);

    var request = new XMLHttpRequest();
    request.open("GET", url, true);

    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            if (request.status == 200) {
                fn(null, JSON.parse(request.responseText)['shorturl']);
            } else {
                fn("connection failure", null);
            }
        }
    }

    request.send();

    // function setResponse(shorturl) {
        // while(result.firstChild) {
            // result.removeChild(result.firstChild);
        // }

        // var link = document.createElement("a");
        // link.href = link.textContent = shorturl;

        // result.textContent = "short url: ";
        // result.appendChild(link);
    // }
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
        version: version,
        separate_output: true
    };

    send("/format.json", obj, function(status, res) {
        if (status == 200) {
            fn(null, res);
        } else {
            fn({ message: "connection failure" }, null);
        }
    });
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
 * Fetch the DOM elements:
 */

var editorDiv = document.getElementById("editor");
var moreOptions = document.getElementById("moreOptions");
var evaluate_button = document.getElementById("evaluate");
var asm_button = document.getElementById("asm");
var ir_button = document.getElementById("ir");
var format_button = document.getElementById("format");
var share_button = document.getElementById("share");
var resultDiv = document.getElementById("result");

/**
 * Initialize the dropdowns:
 */

var exampleDropdown = new Dropdown(document.getElementById("examples"));
var optimizeDropdown = new Dropdown(document.getElementById("optimize"));
var versionDropdown = new Dropdown(document.getElementById("version"));

/**
 * Initialize the Ace editor.
 */

var editor = ace.edit("editor");
var session = editor.getSession();
var Range = ace.require('ace/range').Range;
// Stores ACE editor markers (highights) for errors
var markers = [];

var colors = {
    success: '#E2EEF6',
    error: '#F6E2E2',
    warning: '#FFFBCB'
};

var status = {
    success: 0,
    error: 1,
    warning: 2
};

/**
 * Configure Ace
 */

session.setMode("ace/mode/rust");
editor.setTheme("ace/theme/github");

// Set initial size to match initial content
updateEditorHeight();

/**
 * Initialize the code in the editor. Either fetch the code
 * from the playpen API or from a query parameter.
 */

var query = get_query_parameters();

if ("code" in query) {
    session.setValue(query["code"]);
} else {
    Sample(exampleDropdown.index).set(function(err, text) {
        if (!err) {
            session.setValue(text);
        }
    });
}

/**
 * Listen to the changes to the example dropdown. When a new
 * option is selected, fetch the sample and display it in the editor.
 */

exampleDropdown.change(function(value, index) {
    Sample(index).set(function(err, text) {
        if (!err) {
            session.setValue(text);
        }
    });
});


function handleResult(result) {
    // IR/asm is put in the `result` property.
    if (result.program || result.result) {
        resultDiv.style.backgroundColor = colors.success;
        resultDiv.innerHTML = (result.program || result.result).replace(/\n/g, '<br />');
    } else if (result.rustc && result.rustc.indexOf('error:') !== -1) {
        resultDiv.style.backgroundColor = colors.error;
        handleProblem(result.rustc, "error");
    } else if (result.rustc && result.rustc.indexOf('warning:') !== -1) {
        resultDiv.style.backgroundColor = colors.warning;
        handleProblem(result.rustc, "warning");
    }
}

share_button.onclick = function() {
    share(versionDropdown.selected, session.getValue(), function(err, link) {
        if (err) {
            resultDiv.style.backgroundColor = colors.error;
            resultDiv.innerHTML = err;
        } else {
            resultDiv.style.backgroundColor = colors.success;
            resultDiv.innerHTML = '<a href="' + link + '">'+link+'</a>';
        }
    });
}

evaluate_button.onclick = function() {
    // Note: Playpen expects these to be a string, so we need to coerce the
    // integer.
    var optimize = optimizeDropdown.index + '';

    // clear previous markers, if any
    markers.map(function(id) { editor.getSession().removeMarker(id); });

    evaluate(session.getValue(), versionDropdown.selected, optimize, function(err, result) {
        if (err) {
            resultDiv.style.backgroundColor = colors.error;
            resultDiv.innerHTML = err;
        } else {
            handleResult(result);
        }
    });
};

asm_button.onclick = function() {
    var optimize = optimizeDropdown.index + '';

    // clear previous markers, if any
    markers.map(function(id) { editor.getSession().removeMarker(id); });

    compile("asm",
            session.getValue(),
            versionDropdown.selected,
            optimize,
            function(err, response) {
        if (err) {
            resultDiv.style.backgroundColor = colors.error;
            resultDiv.innerHTML = err;
        } else {
            handleResult(response);
        }
    });
};

ir_button.onclick = function() {
    var optimize = optimizeDropdown.index + '';

    // clear previous markers, if any
    markers.map(function(id) { editor.getSession().removeMarker(id); });

    compile("ir", session.getValue(), versionDropdown.selected, optimize, function(err, response) {
        if (err) {
            resultDiv.style.backgroundColor = colors.error;
            resultDiv.innerHTML = err;
        } else {
            handleResult(response);
        }
    });
};

format_button.onclick = function() {
    format(session.getValue(), versionDropdown.selected, function(err, text) {
        if (err) {
            resultDiv.style.backgroundColor = colors.error;
            resultDiv.innerHTML = err;
        } else {
            session.setValue(text.result);
        }
    });
};

// Called on unsuccessful program run. Detects and prints problems (either
// warnings or errors) in program output and highlights relevant lines and text
// in the code.
function handleProblem(message, problem) {
  // Getting list of ranges with problems
  var lines = message.split(newLineRegex);

  // Cleaning up the message: keeps only relevant problem output
  var cleanMessage = lines.map(function(line) {
    if (line.startsWith("<anon>") || line.indexOf("^") !== -1) {
      var errIndex = line.indexOf(problem + ": ");
      if (errIndex !== -1) return line.slice(errIndex);
      return "";
    }

    // Discard playpen messages, keep the rest
    if (line.startsWith("playpen:")) return "";
    return line;
  }).filter(function(line) {
    return line !== "";
  }).map(function(line) {
    return escapeHTML(line);
  }).join("<br />");

  // Setting message
  resultDiv.innerHTML = cleanMessage;

  // Highlighting the lines
  var ranges = parseProblems(lines);
  markers = ranges.map(function(range) {
    return editor.getSession().addMarker(range, "ace-" + problem + "-line",
      "fullLine", false);
  });

  // Highlighting the specific text
  markers = markers.concat(ranges.map(function(range) {
    return editor.getSession().addMarker(range, "ace-" + problem + "-text",
      "text", false);
  }));
}

// Parses a problem message returning a list of ranges (row:col, row:col) where
// problems in the code have occured.
function parseProblems(lines) {
  var ranges = [];
  for (var i in lines) {
    var line = lines[i];
    if (line.startsWith("<anon>:") && line.indexOf(": ") !== -1) {
      var parts = line.split(/:\s?|\s+/, 5).slice(1, 5);
      var ip = parts.map(function(p) { return parseInt(p, 10) - 1; });
      ranges.push(new Range(ip[0], ip[1], ip[2], ip[3]));
    }
  }

  return ranges;
}

// Keep track of the more options state:
var moreOptionsState = 'more';

/**
 * Handle the more options click events:
 */

moreOptions.onclick = function(event) {
    var options = document.getElementById("options");
    if (moreOptionsState === 'more') {
        options.style.display = 'inline-block';
        moreOptions.innerHTML = 'Hide Options';
        moreOptionsState = 'less';
    } else {
        options.style.display = 'none';
        moreOptions.innerHTML = 'More Options <i class="icon-angle-down"></i>';
        moreOptionsState = 'more';
    }
}

// Changes the height of the editor to match its contents
function updateEditorHeight() {
    // http://stackoverflow.com/questions/11584061/
    var newHeight = editor.getSession().getScreenLength()
        * editor.renderer.lineHeight
        + editor.renderer.scrollBar.getWidth();

    editorDiv.style.height = Math.ceil(newHeight).toString() + "px";
    editor.resize();
};

// Called on unsuccessful program run. Detects and prints problems (either
// warnings or errors) in program output and highlights relevant lines and text
// in the code.
function handleProblem(message, problem) {
    // Getting list of ranges with problems
    var lines = message.split(newLineRegex);

    // Cleaning up the message: keeps only relevant problem output
    var cleanMessage = lines.map(function(line) {

    if (line.startsWith("<anon>") || line.indexOf("^") !== -1) {
        var errIndex = line.indexOf(problem + ": ");
        if (errIndex !== -1) return line.slice(errIndex);
        return "";
    }

    // Discard playpen messages, keep the rest
    if (line.startsWith("playpen:")) return "";
        return line;
    }).filter(function(line) {
        return line !== "";
    }).map(function(line) {
        return escapeHTML(line);
    }).join("<br />");

    // Setting message
    resultDiv.innerHTML = cleanMessage;

    // Highlighting the lines
    var ranges = parseProblems(lines);
    markers = ranges.map(function(range) {
        return editor.getSession().addMarker(range, "ace-" + problem + "-line",
        "fullLine", false);
    });

    // Highlighting the specific text
    markers = markers.concat(ranges.map(function(range) {
        return editor.getSession().addMarker(range, "ace-" + problem + "-text",
        "text", false);
    }));
}

function send(path, data, callback) {
    var request = new XMLHttpRequest();
    request.open("POST", 'http://play.rust-lang.org' + path, true);
    request.setRequestHeader("Content-Type", "application/json");
    request.onreadystatechange = function() {
        if (request.readyState == 4) {
            var obj;
            try {
                obj = JSON.parse(request.response);
            } catch(e) {
                obj = {};
            }

            callback(request.status, obj);
        }
    }
    request.send(JSON.stringify(data));
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

}(window, document));
