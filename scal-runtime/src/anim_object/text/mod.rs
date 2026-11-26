pub(crate) mod atlas;
pub mod code;
pub(crate) mod mesh;
pub(crate) mod pipeline;
pub(crate) mod render;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::{
    anim_object::{
        Alignment, Transform,
        object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult},
        text::code::{Code, highliter::CodeHighlighter},
    },
    types::into_cosmic,
};
use anyhow::{Context, Result};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use glam::Vec2;
use scal_core::{Color, TextModifier, TextSpan};
use tree_sitter_highlight::Highlighter;
use uuid::Uuid;
/// RGBA

/// Fills `buffer` with the text content, using per-span attributes when the text has spans.
fn set_content(
    buffer: &mut Buffer,
    text: &Text,
    align: Option<cosmic_text::Align>,
) {
    let default_attrs = Attrs::new().family(Family::Name(&text.font_family));
    if text.spans.is_empty() {
        buffer.set_text(&text.value, &default_attrs, Shaping::Advanced, align);
        return;
    }
    let spans = text.spans.iter().map(|span| {
        let mut attrs = default_attrs.clone();
        if let Some(color) = span.color {
            attrs = attrs.color(into_cosmic(color));
        }
        if span.bold {
            attrs = attrs.weight(Weight::BOLD);
        }
        (span.text.as_str(), attrs)
    });
    buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, align);
}

fn measure_text_width(text: &Text) -> f32 {
    static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();
    let mut font_system = FONT_SYSTEM
        .get_or_init(|| Mutex::new(new_font_system()))
        .lock()
        .expect("font system lock poisoned");

    let metrics = Metrics::new(text.font_size, text.font_size * 1.2);
    let mut buffer = Buffer::new(&mut *font_system, metrics);
    set_content(&mut buffer, text, None);
    buffer.shape_until_scroll(&mut *font_system, false);

    buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0f32, f32::max)
}

impl From<Alignment> for cosmic_text::Align {
    fn from(a: Alignment) -> Self {
        match a {
            Alignment::Start => cosmic_text::Align::Left,
            Alignment::Center => cosmic_text::Align::Center,
            Alignment::End => cosmic_text::Align::Right,
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
    pub align: Alignment,
    pub value: String,
    pub color: Color,
    pub font_size: f32,
    pub transform: Transform,
    pub cached_size: Option<Vec2>,
    pub modifications: Vec<TextModifier>,
    pub spans: Vec<TextSpan>,
    /// The number the `{:spec}` placeholders of `template` currently show.
    pub number: f32,
    /// Text with `{:spec}` placeholders; `value` and `spans` hold it rendered with `number`.
    /// `None` when the text has no placeholder.
    pub template: Option<NumberTemplate>,
}

/// The unrendered content of a `Text` that shows an animated number.
#[derive(Clone, Debug)]
pub struct NumberTemplate {
    pub value: String,
    pub spans: Vec<String>,
}

impl Text {
    /// Remembers the current content as a template if it has `{:spec}` placeholders and renders
    /// it with `number`.
    pub fn with_number(mut self, number: f32) -> Self {
        use scal_core::number_format::has_placeholder;
        let has_any = has_placeholder(&self.value)
            || self.spans.iter().any(|s| has_placeholder(&s.text));
        if has_any {
            self.template = Some(NumberTemplate {
                value: self.value.clone(),
                spans: self.spans.iter().map(|s| s.text.clone()).collect(),
            });
            self.number = number;
            self.render_template();
        }
        self
    }

    /// Re-renders `value`/`spans` from the template. Returns `true` if the text changed.
    fn render_template(&mut self) -> bool {
        use scal_core::number_format::format_template;
        let Some(template) = &self.template else {
            return false;
        };
        let value = format_template(&template.value, self.number);
        let mut changed = value != self.value;
        for (span, source) in self.spans.iter_mut().zip(&template.spans) {
            let text = format_template(source, self.number);
            changed |= text != span.text;
            span.text = text;
        }
        if changed {
            self.value = value;
            self.cached_size = None;
        }
        changed
    }
}

impl AnimObjectTrait for Text {
    fn number(&self) -> Option<f32> {
        self.template.as_ref().map(|_| self.number)
    }
    fn set_number(&mut self, number: f32) -> Result<bool> {
        anyhow::ensure!(
            self.template.is_some(),
            "text {:?} has no {{:spec}} placeholder to show a number in",
            self.value
        );
        self.number = number;
        Ok(self.render_template())
    }
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
        let width = if self.value.is_empty() {
            1.0
        } else {
            measure_text_width(self)
        };
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

/// Creates a [`FontSystem`] without rescanning the system fonts every time.
///
/// `FontSystem::new()` scans every installed font, which takes a long time (especially in
/// debug builds) and every `Animator` needs one, so a reload would pay for it twice. The
/// scanned font database is kept around and cloned for each new `FontSystem` instead.
pub fn new_font_system() -> FontSystem {
    use cosmic_text::fontdb;
    static FONT_DB: OnceLock<(String, fontdb::Database)> = OnceLock::new();
    let started = std::time::Instant::now();
    let mut scanned = false;
    let (locale, db) = FONT_DB.get_or_init(|| {
        scanned = true;
        FontSystem::new().into_locale_and_db()
    });
    let system = FontSystem::new_with_locale_and_db(locale.clone(), db.clone());
    log::debug!(
        "font system ready in {:.1?} ({})",
        started.elapsed(),
        if scanned { "scanned system fonts" } else { "reused font database" }
    );
    system
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
            font_system: new_font_system(),
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
            Some(cosmic_text::Align::from(code.align)),
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

        set_content(&mut buffer, text, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let max_width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0f32, f32::max);

        buffer.set_size(Some(max_width), None);

        set_content(
            &mut buffer,
            text,
            Some(cosmic_text::Align::from(text.align)),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        buffer
    }
}
