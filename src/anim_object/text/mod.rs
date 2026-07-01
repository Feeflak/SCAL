pub(crate) mod atlas;
pub(crate) mod mesh;
pub(crate) mod pipeline;
pub(crate) mod render;

use crate::types::*;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
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
    pub font_system: FontSystem,
    pub atlas: atlas::GlyphAtlas,
}

impl TextManager {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            atlas: atlas::GlyphAtlas::new(),
        }
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
