use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Syntax, color::Color, highlight_specs};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Base16 {
    pub colors: [Color; 16],
}

impl Base16 {
    #[must_use]
    pub fn from_hex(hex: [u32; 16]) -> Self {
        let mut colors = [Color::BLACK; 16];
        for (i, &h) in hex.iter().enumerate() {
            colors[i] = Color::from(h);
        }
        Self { colors }
    }
}

/// A syntax highlighting theme, constructed from a Base16 palette.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub base: Base16,
    pub syntax_specific: HashMap<Syntax, SyntaxTheme>,
}
impl Theme {
    #[must_use]
    pub fn from_base16(base: Base16) -> Self {
        let mut syntax_specific = HashMap::new();

        for (syntax, spec) in highlight_specs::all_specs() {
            let theme = SyntaxTheme::from_base16_and_spec(base, spec);
            syntax_specific.insert(syntax, theme);
        }

        Self {
            base,
            syntax_specific,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_base16(Base16::default())
    }
}

impl Default for Base16 {
    fn default() -> Self {
        Self {
            colors: [
                Color {
                    r: 0.067,
                    g: 0.071,
                    b: 0.114,
                    a: 1.0,
                },
                Color {
                    r: 0.102,
                    g: 0.106,
                    b: 0.165,
                    a: 1.0,
                },
                Color {
                    r: 0.129,
                    g: 0.133,
                    b: 0.204,
                    a: 1.0,
                },
                Color {
                    r: 0.157,
                    g: 0.173,
                    b: 0.204,
                    a: 1.0,
                },
                Color {
                    r: 0.290,
                    g: 0.314,
                    b: 0.341,
                    a: 1.0,
                },
                Color {
                    r: 0.627,
                    g: 0.659,
                    b: 0.804,
                    a: 1.0,
                },
                Color {
                    r: 0.627,
                    g: 0.659,
                    b: 0.804,
                    a: 1.0,
                },
                Color {
                    r: 0.627,
                    g: 0.659,
                    b: 0.804,
                    a: 1.0,
                },
                Color {
                    r: 0.933,
                    g: 0.427,
                    b: 0.522,
                    a: 1.0,
                },
                Color {
                    r: 0.965,
                    g: 0.584,
                    b: 0.357,
                    a: 1.0,
                },
                Color {
                    r: 0.843,
                    g: 0.651,
                    b: 0.373,
                    a: 1.0,
                },
                Color {
                    r: 0.584,
                    g: 0.773,
                    b: 0.380,
                    a: 1.0,
                },
                Color {
                    r: 0.220,
                    g: 0.659,
                    b: 0.616,
                    a: 1.0,
                },
                Color {
                    r: 0.443,
                    g: 0.600,
                    b: 0.933,
                    a: 1.0,
                },
                Color {
                    r: 0.643,
                    g: 0.522,
                    b: 0.867,
                    a: 1.0,
                },
                Color {
                    r: 0.467,
                    g: 0.204,
                    b: 0.251,
                    a: 1.0,
                },
            ],
        }
    }
}

pub struct HighlightSpec {
    pub names: Vec<String>,
    pub indices: Vec<usize>, // indices into Base16 palette
}
impl HighlightSpec {
    pub fn new(names: Vec<&str>, indices: Vec<usize>) -> Self {
        Self {
            indices,
            names: names.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTheme {
    pub highlight_names: Vec<String>,
    pub highlight_colors: Vec<Color>,
    pub default_color: Color,
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
            default_color: base.colors[5],
        }
    }

    pub fn color_for_name(&self, name: &str) -> Option<Color> {
        self.highlight_names
            .iter()
            .position(|n| n == name)
            .and_then(|idx| self.highlight_colors.get(idx).copied())
    }
}
