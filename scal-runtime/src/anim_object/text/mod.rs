pub(crate) mod atlas;
pub mod code;
pub(crate) mod mesh;
pub(crate) mod pipeline;
pub(crate) mod render;

use std::collections::HashMap;

use crate::{
    anim_object::{
        Transform,
        object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult},
        text::code::{Code, highliter::CodeHighlighter},
    },
    types::into_cosmic,
};
use anyhow::{Context, Result};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
use glam::Vec2;
use scal_core::{Color, TextModifier};
use tree_sitter_highlight::Highlighter;
use uuid::Uuid;
/// RGBA

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Align {
    Left,
    Right,
    Center,
    Justified,
    End,
}
impl Into<cosmic_text::Align> for Align {
    fn into(self) -> cosmic_text::Align {
        match self {
            Align::Left => cosmic_text::Align::Left,
            Align::Right => cosmic_text::Align::Right,
            Align::Center => cosmic_text::Align::Center,
            Align::Justified => cosmic_text::Align::Justified,
            Align::End => cosmic_text::Align::End,
        }
    }
}
#[derive(Clone, Debug)]
pub enum FontSpec {
    Family(String),
    Named(String),
}
#[derive(Clone, Debug)]
pub struct Text {
    pub id: Uuid,
    pub font_family: String,
    pub alignment: Align,
    pub value: String,
    pub color: Color,
    pub font_size: f32,
    pub transform: Transform,
    pub cached_size: Option<Vec2>,
    pub modifications: Vec<TextModifier>,
}

impl AnimObjectTrait for Text {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        if let Some(cached) = self.cached_size {
            return cached;
        }
        let line_count = self.value.lines().count().max(1);
        let height = line_count as f32 * self.font_size * 1.2;
        let width = self.value.len() as f32 * self.font_size * 0.5;
        glam::vec2(width, height)
    }
    fn generate_mesh(&mut self, mgr: &mut TextManager) -> MeshResult {
        Ok(crate::anim_object::text::mesh::generate_text_mesh(
            mgr, self,
        ))
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

pub struct TextManager {
    pub code_highlighter: CodeHighlighter,
    pub font_system: FontSystem,
    pub atlas: atlas::GlyphAtlas,
    pub layouts: HashMap<Uuid, Buffer>,
    pub scale: f32,
}

impl TextManager {
    pub fn new(scale: f32) -> Self {
        Self {
            layouts: HashMap::new(),
            code_highlighter: CodeHighlighter {
                highlighter: Highlighter::new(),
            },
            font_system: FontSystem::new(),
            atlas: atlas::GlyphAtlas::new(scale),
            scale,
        }
    }
    pub fn layout_code(&mut self, code: &mut Code, id: Uuid) -> Result<Buffer> {
        if !code.dirty {
            if let Some(buffer) = self.layouts.get(&id) {
                return Ok(buffer.to_owned());
            }
        }
        code.dirty = false;
        let metrics = Metrics::new(code.font_size, code.font_size * 1.2);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let default_attrs = Attrs::new().family(Family::Name(&code.font_family));

        // Keep the backing strings alive while set_rich_text consumes them.
        let mut owned_spans: Vec<(String, Attrs)> = Vec::new();

        let line_count = code.lines.len();
        let gutter_digits = if line_count > 0 {
            (line_count.ilog10() + 1) as usize
        } else {
            1
        };
        for (line_index, line) in code.lines.iter().enumerate() {
            if code.show_line_numbers {
                let num_str = format!("{:>w$} ", line_index + 1, w = gutter_digits);
                owned_spans.push((
                    num_str,
                    Attrs::new()
                        .family(Family::Name(&code.font_family))
                        .color(into_cosmic(code.line_number_color)),
                ));
            }
            for span in &line.spans {
                owned_spans.push((
                    span.value.clone(),
                    Attrs::new()
                        .family(Family::Name(&code.font_family))
                        .color(into_cosmic(span.color)),
                ));
            }
        }

        let rich_spans = owned_spans
            .iter()
            .map(|(text, attrs)| (text.as_str(), attrs.clone()));

        buffer.set_rich_text(
            rich_spans,
            &default_attrs,
            Shaping::Advanced,
            Some(code.alignment.into()),
        );

        buffer.shape_until_scroll(&mut self.font_system, false);

        self.layouts.insert(id, buffer);
        self.layouts
            .get(&id)
            .context("layout was just inserted, should exist")
            .map(|b| b.to_owned())
    }

    pub fn layout(&mut self, text: &Text) -> Buffer {
        let metrics = Metrics::new(text.font_size, text.font_size * 1.2);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let attrs = Attrs::new().family(cosmic_text::Family::Name(&text.font_family));

        buffer.set_text(
            &text.value,
            &attrs,
            Shaping::Advanced,
            Some(text.alignment.into()),
        );

        const PRUNE: bool = false;
        buffer.shape_until_scroll(&mut self.font_system, PRUNE);

        buffer
    }
}
