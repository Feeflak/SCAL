use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Syntax, color::Color, highlight_specs};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
/// <https://github.com/tinted-theming/schemes/tree/spec-0.11/base16>
#[allow(missing_docs)]
pub struct Base16 {
    pub colors: [Color; 16],
}

impl Base16 {
    #[must_use]
    /// <https://github.com/tinted-theming/schemes/tree/spec-0.11/base16>
    /// Use normal hex colors and just remove # from the front and replace it with 0x
    /// ```ignore
    /// let theme = Theme::from_base16(Base16::from_hex([
    ///     0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
    ///     0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    /// ]));
    /// ```
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
    /// <https://github.com/tinted-theming/schemes/tree/spec-0.11/base16>
    pub base: Base16,

    #[allow(missing_docs)]
    pub syntax_specific: HashMap<Syntax, SyntaxTheme>,
}
impl Theme {
    #[must_use]
    #[allow(missing_docs)]
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
/// Highlight specification used for syntax highlighting using Tree-sitter.
pub struct HighlightSpec {
    /// Names of the Tree-sitter nodes. Syntax specific
    pub names: Vec<String>,
    /// indices into Base16 palette
    pub indices: Vec<usize>,
}
#[allow(missing_docs)]
impl HighlightSpec {
    pub fn new(names: Vec<&str>, indices: Vec<usize>) -> Self {
        Self {
            indices,
            names: names
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Syntax specific syntax highlighting theme settings
pub struct SyntaxTheme {
    /// Names of the Tree-sitter nodes. Syntax specific
    pub highlight_names: Vec<String>,
    /// indices into Base16 palette
    pub highlight_colors: Vec<Color>,
    #[allow(missing_docs)]
    pub default_color: Color,
}

#[allow(missing_docs)]
impl SyntaxTheme {
    #[must_use]
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

    #[must_use]
    pub fn color_for_name(&self, name: &str) -> Option<Color> {
        self.highlight_names
            .iter()
            .position(|n| n == name)
            .and_then(|idx| self.highlight_colors.get(idx).copied())
    }
}
