use crate::anim_object::text::code::{Syntax, theme::HighlightSpec};

pub fn all_specs() -> Vec<(Syntax, HighlightSpec)> {
    vec![
        (Syntax::Rust, rust_spec()),
        (Syntax::Nix, nix_spec()),
        (Syntax::JS, js_spec()),
        (Syntax::Zig, zig_spec()),
        (Syntax::Python, python_spec()),
    ]
}
pub fn rust_spec() -> HighlightSpec {
    HighlightSpec {
        names: vec![
            "attribute",
            "comment",
            "constant",
            "constant.builtin",
            "constructor",
            "embedded",
            "escape",
            "function",
            "function.builtin",
            "function.macro",
            "keyword",
            "keyword.function",
            "keyword.operator",
            "keyword.return",
            "label",
            "lifetime",
            "macro",
            "method",
            "module",
            "namespace",
            "number",
            "operator",
            "property",
            "field",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "punctuation.special",
            "string",
            "string.special",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
            "self",
        ],

        indices: vec![
            12, // attribute
            3,  // comment
            8,  // constant          (was 11 — dark orange for enum variants like TypeWriter)
            8,  // constant.builtin
            9,  // constructor       (was 14 — light orange like types)
            5,  // embedded
            10, // escape
            13, // function
            13, // function.builtin
            8,  // function.macro      (was 13 — red)
            14, // keyword           (was 8 — purple)
            14, // keyword.function
            14, // keyword.operator
            14, // keyword.return
            12, // label
            14, // lifetime          (was 8 — purple)
            8,  // macro               (was 13 — red)
            13, // method
            10, // module            (was 14 — light orange/almost yellow)
            10, // namespace         (was 14 — light orange/almost yellow)
            9,  // number            (dark orange)
            15, // operator
            5,  // property
            5,  // field
            15, // punctuation
            5,  // punctuation.bracket
            15, // punctuation.delimiter
            15, // punctuation.special
            11, // string            (was 10 — green)
            11, // string.special
            10, // type              (was 14 — light orange/almost yellow)
            10, // type.builtin      (was 14 — light orange/almost yellow)
            8,  // variable            (was 5 — red for let bindings)
            5,  // variable.builtin
            5,  // variable.parameter  (kept gray for fn params like `n`)
            5,  // self
        ],
    }
}

pub fn nix_spec() -> HighlightSpec {
    HighlightSpec {
        names: vec![
            "comment",
            "keyword",
            "function",
            "builtin",
            "string",
            "number",
            "attribute",
            "variable",
            "operator",
            "punctuation",
            "path",
            "boolean",
            "null",
        ],
        indices: vec![
            3,  // comment
            8,  // keyword
            13, // function
            11, // builtin
            10, // string
            9,  // number
            6,  // attribute
            5,  // variable
            7,  // operator
            4,  // punctuation
            6,  // path
            11, // boolean
            5,  // null
        ],
    }
}
pub fn python_spec() -> HighlightSpec {
    HighlightSpec {
        names: vec![
            "comment",
            "keyword",
            "function",
            "builtin",
            "type",
            "string",
            "number",
            "constant",
            "variable",
            "attribute",
            "operator",
            "punctuation",
            "decorator",
            "exception",
            "boolean",
            "none",
        ],
        indices: vec![
            3,  // comment
            8,  // keyword
            13, // function
            13, // builtin
            14, // type
            10, // string
            9,  // number
            11, // constant
            5,  // variable
            6,  // attribute
            7,  // operator
            4,  // punctuation
            12, // decorator
            8,  // exception
            11, // boolean
            5,  // none
        ],
    }
}
pub fn js_spec() -> HighlightSpec {
    HighlightSpec {
        names: vec![
            "comment",
            "keyword",
            "function",
            "method",
            "class",
            "string",
            "number",
            "constant",
            "variable",
            "property",
            "operator",
            "punctuation",
            "tag",
            "attribute",
            "boolean",
            "null",
            "regex",
        ],
        indices: vec![
            3,  // comment
            8,  // keyword
            13, // function
            13, // method
            14, // class (mapped to type-ish color slot)
            10, // string
            9,  // number
            11, // constant
            5,  // variable
            6,  // property
            7,  // operator
            4,  // punctuation
            12, // tag (jsx/html-like)
            6,  // attribute
            11, // boolean
            5,  // null
            10, // regex
        ],
    }
}
pub fn zig_spec() -> HighlightSpec {
    HighlightSpec {
        names: vec![
            "comment",
            "keyword",
            "function",
            "type",
            "string",
            "number",
            "constant",
            "variable",
            "field",
            "operator",
            "punctuation",
            "builtin",
            "boolean",
            "null",
            "error",
        ],
        indices: vec![
            3,  // comment
            8,  // keyword
            13, // function
            14, // type
            10, // string
            9,  // number
            11, // constant
            5,  // variable
            6,  // field
            7,  // operator
            4,  // punctuation
            11, // builtin
            11, // boolean
            5,  // null
            8,  // error
        ],
    }
}
