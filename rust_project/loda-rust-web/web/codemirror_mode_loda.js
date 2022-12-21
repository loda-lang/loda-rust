// Syntax coloring for LODA assembly language
// https://github.com/loda-lang/loda-lang
(function(mod) {
    if (typeof exports == "object" && typeof module == "object") // CommonJS
      mod(require("../../lib/codemirror"), require("../../addon/mode/simple"));
    else if (typeof define == "function" && define.amd) // AMD
      define(["../../lib/codemirror", "../../addon/mode/simple"], mod);
    else // Plain browser env
      mod(CodeMirror);
  })(function(CodeMirror) {
    "use strict";
  
    CodeMirror.defineSimpleMode("loda", {
      start: [
        {regex: /[$][$]\d+/, token: "number"},
        {regex: /[$]\d+/, token: "number"},
        {regex: /-?\d+/, token: "number"},
        {regex: /\s*(?:mov|add|sub|trn|mul|div|dif|mod|pow|gcd|bin|cmp|min|max|lpb|lpe|clr|seq|lps|f\d\d)\b/,
          token: "keyword", sol: true },
        {regex: /;.*/, token: "comment"},
      ],
    });
      
    CodeMirror.defineMIME("text/x-loda", "loda");
});
