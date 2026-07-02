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
            "comment",
            "keyword",
            "keyword.function",
            "function",
            "method",
            "type",
            "string",
            "number",
            "constant",
            "constant.builtin",
            "variable",
            "variable.builtin",
            "property",
            "operator",
            "punctuation",
            "attribute",
            "macro",
            "lifetime",
            "self",
        ],
        indices: vec![
            3,  // comment
            8,  // keyword
            8,  // keyword.function
            13, // function
            13, // method
            14, // type
            10, // string
            9,  // number
            11, // constant
            11, // constant.builtin
            5,  // variable
            5,  // variable.builtin
            6,  // property
            7,  // operator
            4,  // punctuation
            12, // attribute
            13, // macro
            8,  // lifetime
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
