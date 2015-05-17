"use strict";

// For convenience of development
var PREFIX = location.href.indexOf("/web.html") != -1 ? "https://play.rust-lang.org/" : "/";

var samples = 2;

function optionalLocalStorageGetItem(key) {
    try {
        return localStorage.getItem(key);
    } catch(e) {
        return null;
    }
}

function optionalLocalStorageSetItem(key, value) {
    try {
        localStorage.setItem(key, value);
    } catch(e) {
        // ignore
    }
}

function build_themes(themelist) {
    // Load all ace themes, sorted by their proper name.
    var themes = themelist.themes;
    themes.sort(function (a, b) {
        if (a.caption < b.caption) {
            return -1
        } else if (a.caption > b.caption) {
            return 1;
        }
        return 0;
    });

    var themeopt,
        themefrag = document.createDocumentFragment();
    for (var i=0; i < themes.length; i++) {
        themeopt = document.createElement("option");
        themeopt.setAttribute("val", themes[i].theme);
        themeopt.textContent = themes[i].caption;
        themefrag.appendChild(themeopt);
    }
    document.getElementById("themes").appendChild(themefrag);
}

function send(path, data, callback, button, message, result) {
    button.disabled = true;

    set_result(result, "<p class=message>" + message);

    var request = new XMLHttpRequest();
    request.open("POST", PREFIX + path, true);
    request.setRequestHeader("Content-Type", "application/json");
    request.onreadystatechange = function() {
        button.disabled = false;
        if (request.readyState == 4) {
            var json;

            try {
                json = JSON.parse(request.response);
            } catch (e) {
                console.log("JSON.parse(): " + e);
            }

            if (request.status == 200) {
                callback(json);
            } else {
                set_result(result, "<p class=error>Connection failure" +
                    "<p class=error-explanation>Are you connected to the Internet?");
            }
        }
    }
    request.timeout = 10000;
    request.ontimeout = function() {
        set_result(result, "<p class=error>Connection timed out" +
            "<p class=error-explanation>Are you connected to the Internet?");
    }
    request.send(JSON.stringify(data));
}

var PYGMENTS_TO_ACE_MAPPINGS = {
    'asm': {
        'c':  'ace_comment', // Comment,
        'na': 'ace_support ace_function ace_directive', // Name.Attribute,
        'no': 'ace_constant', // Name.Constant,
        'nl': 'ace_entity ace_name ace_function', // Name.Label,
        'nv': 'ace_variable ace_parameter ace_register', // Name.Variable,
        'mh': 'ace_constant ace_character ace_hexadecimal', // Number.Hex,
        'mi': 'ace_constant ace_character ace_decimal', // Number.Integer,
        'p':  'ace_punctuation', // Punctuation,
        's':  'ace_string', // String,
        'sc': 'ace_string', // String.Char,
        '':   '', // Text,
    },
    'llvm-ir': {
        'c':            'ace_comment', // Comment
        'k':            'ace_keyword', // Keyword
        'kt':           'ace_storage ace_type', // Keyword.Type
        'nl':           'ace_identifier', // Name.Label
        'nv':           'ace_variable', // Name.Variable
        'nv-Anonymous': 'ace_support ace_variable', // Name.Variable.Anonymous
        'vg':           'ace_variable ace_other', // Name.Variable.Global
        'm':            'ace_constant ace_numeric', // Number
        'p':            'ace_punctuation', // Punctuation
        's':            'ace_string', // String
        '':             '', // Text
    }
};

function rehighlight(pygmentized, language) {
    var mappings = PYGMENTS_TO_ACE_MAPPINGS[language];
    return pygmentized.replace(/<span class="([^"]*)">([^<]*)<\/span>/g, function() {
        var classes = mappings[arguments[1]];
        if (classes) {
            return '<span class="' + classes + '">' + arguments[2] + '</span>';
        } else {
            return arguments[2];
        }
    });
}

function redrawResult(result) {
    // Sadly the fun letter-spacing animation can leave artefacts,
    // so we want to manually trigger a redraw. It doesn’t matter
    // whether it’s relative or static for now, so we’ll flip that.
    result.parentNode.style.visibility = "hidden";
    result.parentNode.offsetHeight;
    result.parentNode.style.visibility = "";
}

function evaluate(result, code, version, optimize, button) {
    send("evaluate.json", {code: code, version: version, optimize: optimize, separate_output: true},
        function(object) {
            var samp = document.createElement("samp");
            samp.className = ("program" in object) ? "rustc-warnings" : "rustc-errors";
            samp.textContent = object.rustc;
            var pre = document.createElement("pre");
            pre.appendChild(samp);
            set_result(result, pre);
            if ("program" in object) {
                var samp = document.createElement("samp");
                samp.className = "output";
                samp.textContent = object.program;
                var pre = document.createElement("pre");
                pre.appendChild(samp);
                result.appendChild(pre);

                var div = document.createElement("p");
                div.className = "message";
                div.textContent = "Program ended.";
                result.appendChild(div);
            }
    }, button, "Running…", result);
}

function compile(emit, result, code, version, optimize, button) {
    send("compile.json", {emit: emit, code: code, version: version, optimize: optimize,
                          highlight: true}, function(object) {
        if ("error" in object) {
            set_result(result, "<pre class=highlight><samp class=rustc-errors></samp></pre>");
            result.firstChild.firstChild.textContent = object["error"];
        } else {
            set_result(result, "<pre class=highlight><code>" + rehighlight(object["result"], emit) + "</code></pre>");
        }
    }, button, "Compiling…", result);
}

function format(result, session, version, button) {
    send("format.json", {code: session.getValue(), version: version}, function(object) {
        if ("error" in object) {
            set_result(result, "<pre class=highlight><samp class=rustc-errors></samp></pre>");
            result.firstChild.firstChild.textContent = object["error"];
        } else {
            clear_result();
            session.setValue(object["result"]);
        }
    }, button, "Formatting…", result);
}

function httpRequest(method, url, data, expect, on_success, on_fail) {
    var req = new XMLHttpRequest();

    req.open(method, url, true);
    req.onreadystatechange = function() {
        if (req.readyState == XMLHttpRequest.DONE) {
            if (req.status == expect) {
                if (on_success) {
                    on_success(req.responseText);
                }
            } else {
                if (on_fail) {
                    on_fail(req.status, req.responseText);
                }
            }
        }
    };

    if (method === "GET") {
        req.send();
    } else if (method === "POST") {
        req.send(data);
    }
}

function share(result, version, code, button) {
    var playurl = "https://play.rust-lang.org?code=" + encodeURIComponent(code);
    playurl += "&version=" + encodeURIComponent(version);
    if (playurl.length > 5000) {
        set_result(result, "<p class=error>Sorry, your code is too long to share this way." +
            "<p class=error-explanation>At present, sharing produces a link containing the" +
            " code in the URL, and the URL shortener used doesn’t accept URLs longer than" +
            " <strong>5000</strong> characters. Your code results in a link that is <strong>" +
            playurl.length + "</strong> characters long. Try shortening your code.");
        return;
    }

    var url = "https://is.gd/create.php?format=json&url=" + encodeURIComponent(playurl);

    button.disabled = true;

    set_result(result, "<p>Short URL: ");
    var link = document.createElement("a");
    link.href = link.textContent = playurl;
    link.className = "shortening-link";
    result.firstChild.appendChild(link);

    function repaint() {
        // Sadly the fun letter-spacing animation can leave artefacts in at
        // least Firefox, so we want to manually trigger a repaint. It doesn’t
        // matter whether it’s relative or static for now, so we’ll flip that.
        result.parentNode.style.visibility = "hidden";
        result.parentNode.offsetHeight;
        result.parentNode.style.visibility = "";
    }

    var repainter = setInterval(repaint, 50);
    httpRequest("GET", url, null, 200,
                function(response) {
                    clearInterval(repainter);

                    var link = result.firstChild.firstElementChild;
                    link.className = "";
                    link.href = link.textContent = JSON.parse(response)['shorturl'];

                    repaint();
                },
                function(status, response) {
                    clearInterval(repainter);

                    set_result(result, "<p class=error>Connection failure" +
                        "<p class=error-explanation>Are you connected to the Internet?");

                    repaint();
                }
    );
}
>>>>>>> Add separate HTTP request function and refactor share()

}

function getQueryParameters() {
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

function clear_result(result) {
    result.innerHTML = "";
    result.parentNode.setAttribute("data-empty", "");
	set_result.editor.resize();
}

function set_result(result, contents) {
    result.parentNode.removeAttribute("data-empty");
    if (typeof contents == "string") {
        result.innerHTML = contents;
    } else {
        result.textContent = "";
        result.appendChild(contents);
    }
    set_result.editor.resize();
}

function set_keyboard(editor, mode) {
    if (mode == "Emacs") {
        editor.setKeyboardHandler("ace/keyboard/emacs");
    } else if (mode == "Vim") {
        editor.setKeyboardHandler("ace/keyboard/vim");
        if (!set_keyboard.vim_set_up) {
            ace.config.loadModule("ace/keyboard/vim", function(m) {
                var Vim = ace.require("ace/keyboard/vim").CodeMirror.Vim;
                Vim.defineEx("write", "w", function(cm, input) {
                    cm.ace.execCommand("evaluate");
                });
            });
        }
        set_keyboard.vim_set_up = true;
    } else {
        editor.setKeyboardHandler(null);
    }
}

function set_theme(editor, themelist, theme) {
    var themes = document.getElementById("themes");
    var themepath = null,
        i = 0,
        themelen = themelist.themes.length,
        selected = themes.options[themes.selectedIndex];
    if (selected.textContent === theme) {
        themepath = selected.getAttribute("val");
    } else {
        for (i; i < themelen; i++) {
            if (themelist.themes[i].caption == theme) {
                themes.selectedIndex = i;
                themepath = themelist.themes[i].theme;
                break;
            }
        }
    }
    if (themepath !== null) {
        editor.setTheme(themepath);
        optionalLocalStorageSetItem("theme", theme);
    }
}

function getRadioValue(name) {
    var nodes = document.getElementsByName(name);
    for (var i = 0; i < nodes.length; i++) {
        var node = nodes[i];
        if (node.checked) {
            return node.value;
        }
    }
}

addEventListener("DOMContentLoaded", function() {
    var evaluateButton = document.getElementById("evaluate");
    var asmButton = document.getElementById("asm");
    var irButton = document.getElementById("llvm-ir");
    // var formatButton = document.getElementById("format");
    var shareButton = document.getElementById("share");
    var configureEditorButton = document.getElementById("configure-editor");
    var result = document.getElementById("result").firstChild;
    var clearResultButton = document.getElementById("clear-result");
    var keyboard = document.getElementById("keyboard");
    var themes = document.getElementById("themes");
    var editor = ace.edit("editor");
    set_result.editor = editor;
    var session = editor.getSession();
    var themelist = ace.require("ace/ext/themelist");

    editor.focus();

    build_themes(themelist);

    editor.renderer.on('themeChange', function(e) {
        var path = e.theme;
        ace.config.loadModule(['theme', e.theme], function(t) {
            document.getElementById("result").className = t.cssClass + (t.isDark ? " ace_dark" : "");
        });
    });

    var theme = optionalLocalStorageGetItem("theme");
    if (theme === null) {
        set_theme(editor, themelist, "GitHub");
    } else {
        set_theme(editor, themelist, theme);
    }

    session.setMode("ace/mode/rust");

    var mode = optionalLocalStorageGetItem("keyboard");
    if (mode !== null) {
        set_keyboard(editor, mode);
        keyboard.value = mode;
    }

    var query = getQueryParameters();
    if ("code" in query) {
        session.setValue(query["code"]);
    } else {
        var code = optionalLocalStorageGetItem("code");
        if (code !== null) {
            session.setValue(code);
        }
    }

    if ("version" in query) {
        var radio = document.getElementById("version-" + query.version);
        if (radio !== null) {
            radio.checked = true;
        }
    }

    if (query["run"] === "1") {
        evaluate(result, session.getValue(), getRadioValue("version"),
                 getRadioValue("optimize"), evaluateButton);
    }

    addEventListener("resize", function() {
        editor.resize();
    });

    session.on("change", function() {
        optionalLocalStorageSetItem("code", session.getValue());
    });

    keyboard.onkeyup = keyboard.onchange = function() {
        var mode = keyboard.options[keyboard.selectedIndex].value;
        optionalLocalStorageSetItem("keyboard", mode);
        set_keyboard(editor, mode);
    }

    evaluateButton.onclick = function() {
        evaluate(result, session.getValue(), getRadioValue("version"),
                 getRadioValue("optimize"), evaluateButton);
    };

    editor.commands.addCommand({
        name: "evaluate",
        exec: evaluateButton.onclick,
        bindKey: {win: "Ctrl-Enter", mac: "Ctrl-Enter"}
    });

    // We’re all pretty much agreed that such an obscure command as transposing
    // letters hogging Ctrl-T, normally “open new tab”, is a bad thing.
    var transposeletters = editor.commands.commands.transposeletters;
    editor.commands.removeCommand("transposeletters");
    delete transposeletters.bindKey;
    editor.commands.addCommand(transposeletters);

    asmButton.onclick = function() {
        compile("asm", result, session.getValue(), getRadioValue("version"),
                 getRadioValue("optimize"), asmButton);
    };

    irButton.onclick = function() {
        compile("llvm-ir", result, session.getValue(), getRadioValue("version"),
                 getRadioValue("optimize"), irButton);
    };

    /*
    formatButton.onclick = function() {
        format(result, session, getRadioValue("version"), formatButton);
    };
    */

    shareButton.onclick = function() {
        share(result, getRadioValue("version"), session.getValue(), shareButton);
    };

    configureEditorButton.onclick = function() {
        var dropdown = configureEditorButton.nextElementSibling;
        dropdown.style.display = dropdown.style.display ? "" : "block";
    };

    clearResultButton.onclick = function() {
        clear_result(result);
    };

    themes.onkeyup = themes.onchange = function () {
        set_theme(editor, themelist, themes.selectedOptions[0].text);
    }

}, false);
