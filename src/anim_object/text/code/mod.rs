mod highlight_specs;
pub mod highliter;
pub mod mesh;
pub mod theme;
use anyhow::{Context, Result};
use cosmic_text::Color;
use glam::{Vec2, vec2};
use tree_sitter_highlight::HighlightConfiguration;

use uuid::Uuid;

use std::ops::Range;

use crate::{
    anim_object::Transform,
    anim_object::object_trait::{AnimObj, AnimObjectTrait, BindGroupLoader, MeshResult},
    anim_object::text::{
        Align,
        code::{highliter::CodeHighlighter, theme::Theme},
    },
    anim_op::{AnimOP, AnimationCurve},
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
pub enum CodeHighlightKind {
    Lines { ranges: Vec<Range<usize>> },
    Pattern { regex: String },
}

#[derive(Clone, Debug)]
pub struct CodeHighlight {
    pub color: crate::types::Color,
    pub kind: CodeHighlightKind,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub enum CodeHighlightAction {
    Lines {
        ranges: Vec<Range<usize>>,
        color: crate::types::Color,
        duration: f32,
        curve: AnimationCurve,
    },
    Pattern {
        regex: String,
        color: crate::types::Color,
        duration: f32,
        curve: AnimationCurve,
    },
}
impl CodeHighlightAction {
    pub fn duration_and_curve(&self) -> (f32, AnimationCurve) {
        match self {
            CodeHighlightAction::Lines {
                duration, curve, ..
            } => (*duration, curve.clone()),
            CodeHighlightAction::Pattern {
                duration, curve, ..
            } => (*duration, curve.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Code {
    pub id: Uuid,
    pub transform: Transform,
    pub source_code: String,
    pub theme: Theme,
    pub font_family: String,
    pub alignment: Align,
    pub font_size: f32,
    pub syntax: Syntax,
    pub lines: Vec<TextLine>,
    pub dirty: bool,
    pub padding: f32,
    pub show_line_numbers: bool,
    pub line_number_color: crate::types::Color,
    pub anim_reveal: f32,
    pub anim_spacing: f32,
    pub anim_line_start: usize,
    pub anim_line_end: usize,
    pub anim_style: CodeAnimationStyle,
    pub anim_spacing_accum: f32,
    pub cached_size: Option<Vec2>,
    pub highlights: Vec<CodeHighlight>,
}

#[derive(Clone, Debug)]
pub enum CodeAnimationStyle {
    TypeWriter,
    TypeWriterInstantResize,
    Fold,
}

impl Code {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(AnimObj(Box::new(self.clone())))
    }

    pub fn highlight_lines(
        &self,
        ranges: Vec<Range<usize>>,
        color: crate::types::Color,
        duration: f32,
        curve: AnimationCurve,
    ) -> AnimOP {
        AnimOP::CodeHighlight(
            self.id,
            CodeHighlightAction::Lines {
                ranges,
                color,
                duration,
                curve,
            },
        )
    }

    pub fn highlight_pattern(
        &self,
        regex: String,
        color: crate::types::Color,
        duration: f32,
        curve: AnimationCurve,
    ) -> AnimOP {
        AnimOP::CodeHighlight(
            self.id,
            CodeHighlightAction::Pattern {
                regex,
                color,
                duration,
                curve,
            },
        )
    }

    pub fn clear_highlights(&mut self) {
        self.highlights.clear();
        self.dirty = true;
    }

    pub fn add_lines(
        &self,
        text: &str,
        from_line: usize,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        AnimOP::CodeAddLines(
            self.id,
            text.to_string(),
            from_line,
            duration,
            anim_curve,
            style,
        )
    }

    pub fn modify_line(
        &self,
        line: u32,
        new_text: String,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        AnimOP::CodeModifyLine(self.id, line, new_text, duration, anim_curve, style)
    }

    pub fn remove_lines(
        &self,
        lines: std::ops::Range<u32>,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        AnimOP::CodeRemoveLines(self.id, lines, duration, anim_curve, style)
    }
    pub fn new(
        text: String,
        syntax: Syntax,
        theme: Theme,
        font_family: String,
        alignment: Align,
        font_size: f32,
        transform: Transform,
        padding: f32,
    ) -> Self {
        Self {
            id: transform.uuid,
            transform,
            alignment,
            source_code: text,
            syntax,
            theme,
            font_family,
            font_size,
            lines: vec![],
            dirty: true,
            padding,
            show_line_numbers: false,
            line_number_color: crate::types::Color::new(0.5, 0.5, 0.5, 0.6),
            anim_reveal: 1.0,
            anim_spacing: 0.0,
            anim_line_start: 0,
            anim_line_end: 0,
            anim_style: CodeAnimationStyle::TypeWriter,
            anim_spacing_accum: 0.0,
            cached_size: None,
            highlights: vec![],
        }
    }

    pub fn update_highlight_if_dirty(&mut self, highlighter: &mut CodeHighlighter) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        self.lines.clear();

        let mut spans = highlighter.highlight_code(&self.source_code, &self.theme, &self.syntax)?;
        let mut current_line: Vec<TextSpan> = Vec::new();

        while let Some(span) = spans.pop() {
            if span.value.contains('\n') {
                let parts: Vec<&str> = span.value.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if !part.is_empty() {
                        current_line.push(TextSpan {
                            color: span.color,
                            value: part.to_string(),
                        });
                    }
                    if i < parts.len() - 1 {
                        if !current_line.is_empty() {
                            if let Some(last) = current_line.last_mut() {
                                last.value.push('\n');
                            }
                            self.lines.push(TextLine {
                                spans: std::mem::take(&mut current_line),
                            });
                        }
                    }
                }
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

impl AnimObjectTrait for Code {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        let p = self.padding * 2.0;
        if let Some(cached) = self.cached_size {
            return cached + vec2(p, p);
        }
        let total_lines = self.source_code.lines().count().max(1);
        let animated_count = self.anim_line_end.saturating_sub(self.anim_line_start);
        let base_count = total_lines.saturating_sub(animated_count);
        let visible_animated = (animated_count as f32 * self.anim_reveal) as usize;
        let visible_lines = (base_count + visible_animated).max(1);
        let height = visible_lines as f32 * self.font_size * 1.2;
        let lines: Vec<&str> = self.source_code.lines().collect();
        let max_visible_line_width = lines
            .iter()
            .take(visible_lines)
            .map(|l| l.len())
            .max()
            .unwrap_or(1);
        let mut gutter_width = 0.0;
        if self.show_line_numbers {
            let digits = total_lines.ilog10() as usize + 1;
            gutter_width = (digits as f32 + 1.0) * self.font_size * 0.6;
        }
        let width = max_visible_line_width as f32 * self.font_size * 0.6 + gutter_width;
        glam::vec2(width, height) + vec2(p, p)
    }
    fn generate_mesh(&mut self, mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        crate::anim_object::text::code::mesh::generate_code_mesh(mgr, self.transform.uuid, self)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
