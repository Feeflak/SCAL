use crate::{anim_object::text::code::highlight_specs, types::Color};
use std::collections::HashMap;

use crate::anim_object::text::code::Syntax;
pub struct HighlightSpec {
    pub names: Vec<&'static str>,
    pub indices: Vec<usize>, // indices into Base16 palette
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub highlight_names: Vec<&'static str>,
    pub highlight_colors: Vec<Color>,
}

impl SyntaxTheme {
    pub fn from_base16_and_spec(base: Base16, spec: HighlightSpec) -> Self {
        let mut colors = Vec::with_capacity(spec.indices.len());

        for idx in &spec.indices {
            colors.push(base.colors[*idx]);
        }

        Self {
            highlight_names: spec.names,
            highlight_colors: colors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub syntax_specific: HashMap<Syntax, SyntaxTheme>,
}
impl Theme {
    pub fn from_base16(base: Base16) -> Self {
        let mut syntax_specific = HashMap::new();

        for (syntax, spec) in highlight_specs::all_specs() {
            let theme = SyntaxTheme::from_base16_and_spec(base, spec);
            syntax_specific.insert(syntax, theme);
        }

        Self { syntax_specific }
    }
}

#[derive(Copy, Clone)]
pub struct Base16 {
    pub colors: [Color; 16],
}
