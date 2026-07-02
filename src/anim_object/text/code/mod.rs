mod highlight_specs;
pub mod highliter;
pub mod mesh;
pub mod theme;
use anyhow::{Context, Result};
use cosmic_text::Color;
use tree_sitter_highlight::HighlightConfiguration;

use crate::anim_object::text::{
    Align,
    code::{highliter::CodeHighlighter, theme::Theme},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Syntax {
    Rust,
    Nix,
    Python,
    JS,
    Zig,
}
impl Syntax {
    pub fn language(self) -> tree_sitter::Language {
        match self {
            Syntax::Rust => tree_sitter_rust::LANGUAGE.into(),
            Syntax::Nix => tree_sitter_nix::LANGUAGE.into(),
            Syntax::Python => tree_sitter_python::LANGUAGE.into(),
            Syntax::JS => tree_sitter_javascript::LANGUAGE.into(),
            Syntax::Zig => tree_sitter_zig::LANGUAGE.into(),
        }
    }
    fn highlight_config(&self) -> Result<HighlightConfiguration> {
        match self {
            Syntax::Rust => HighlightConfiguration::new(
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
            .context("while creating rust highlight_config"),
            Syntax::Nix => HighlightConfiguration::new(
                tree_sitter_nix::LANGUAGE.into(),
                "nix",
                tree_sitter_nix::HIGHLIGHTS_QUERY,
                tree_sitter_nix::INJECTIONS_QUERY,
                "",
            )
            .context("while creating nix highlight_config"),
            Syntax::Python => HighlightConfiguration::new(
                tree_sitter_python::LANGUAGE.into(),
                "python",
                tree_sitter_python::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .context("while creating python highlight_config"),

            Syntax::JS => HighlightConfiguration::new(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .context("while creating js highlight_config"),
            Syntax::Zig => HighlightConfiguration::new(
                tree_sitter_zig::LANGUAGE.into(),
                "zig",
                tree_sitter_zig::HIGHLIGHTS_QUERY,
                tree_sitter_zig::INJECTIONS_QUERY,
                "",
            )
            .context("while creating zig highlight_config"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub color: Color,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct TextLine {
    pub spans: Vec<TextSpan>,
}

#[derive(Clone, Debug)]
pub struct Code {
    pub source_code: String,
    pub theme: Theme,
    pub font_family: String,
    pub alignment: Align,
    pub font_size: f32,
    pub syntax: Syntax,
    pub lines: Vec<TextLine>,
    pub dirty: bool,
}

impl Code {
    pub fn new(
        text: String,
        syntax: Syntax,
        theme: Theme,
        font_family: String,
        alignment: Align,
        font_size: f32,
    ) -> Self {
        Self {
            alignment,
            source_code: text,
            syntax,
            theme,
            font_family,
            font_size,
            lines: vec![],
            dirty: true,
        }
    }
    pub fn update_highlight_if_dirty(&mut self, highlighter: &mut CodeHighlighter) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        self.dirty = false;

        self.lines.clear();

        let mut spans = highlighter.highlight_code(&self.source_code, &self.theme, &self.syntax)?;
        let mut current_line = Vec::new();

        while let Some(span) = spans.pop() {
            if span.value.contains('\n') {
                current_line.push(span);

                self.lines.push(TextLine {
                    spans: std::mem::take(&mut current_line),
                });
            } else {
                current_line.push(span);
            }
        }

        if !current_line.is_empty() {
            self.lines.push(TextLine {
                spans: current_line,
            });
        }

        Ok(())
    }
}
