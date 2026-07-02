pub(crate) mod atlas;
pub mod code;
pub(crate) mod mesh;
pub(crate) mod pipeline;
pub(crate) mod render;

use std::collections::HashMap;

use crate::{
    anim_object::text::code::{Code, highliter::CodeHighlighter},
    types::*,
};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
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
    pub font_family: String,
    pub alignment: Align,
    pub value: String,
    pub color: Color,
    pub font_size: f32,
}

pub struct TextManager {
    pub code_highlighter: CodeHighlighter,
    pub font_system: FontSystem,
    pub atlas: atlas::GlyphAtlas,
    pub layouts: HashMap<Uuid, Buffer>,
}

impl TextManager {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
            code_highlighter: CodeHighlighter {
                highlighter: Highlighter::new(),
            },
            font_system: FontSystem::new(),
            atlas: atlas::GlyphAtlas::new(),
        }
    }
    pub fn layout_code(&mut self, code: &Code, id: Uuid) -> Buffer {
        if !code.dirty {
            if let Some(buffer) = self.layouts.get(&id) {
                return buffer.to_owned();
            }
        }
        let metrics = Metrics::new(code.font_size, code.font_size * 1.2);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let default_attrs = Attrs::new().family(Family::Name(&code.font_family));

        // Keep the backing strings alive while set_rich_text consumes them.
        let mut owned_spans: Vec<(String, Attrs)> = Vec::new();

        for (line_index, line) in code.lines.iter().enumerate() {
            for span in &line.spans {
                owned_spans.push((
                    span.value.clone(),
                    Attrs::new()
                        .family(Family::Name(&code.font_family))
                        .color(span.color.into()),
                ));
            }

            // Preserve source line structure
            if line_index + 1 != code.lines.len() {
                owned_spans.push(("\n".to_string(), default_attrs.clone()));
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
        self.layouts.get(&id).unwrap().to_owned()
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
