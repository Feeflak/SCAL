pub mod ansi;

use anyhow::Result;
use glam::{Vec2, Vec3, vec2};
use scal_core::{Color, Ease, Theme};
use uuid::Uuid;

use crate::anim_object::text as text_fn;
use crate::{
    anim_object::{
        Transform, circle,
        compose::{
            Alignment as LayoutAlignment, LayoutBackground, LayoutDir, LayoutItem, LayoutResult,
            PinPoint, layout_with_ids,
        },
        object_trait::{AnimObjectTrait, BindGroupLoader, DynAnimObj, MeshResult},
        render::PipelineKind,
        text::{Align, TextManager},
    },
    anim_op::AnimOperation,
    renderer::{Index, Vertex},
};

/// Runtime terminal emulator window.
pub struct Terminal {
    pub background: DynAnimObj,
    pub text_buffer: TerminalTextBuffer,
    layout_result: LayoutResult,
    pub close_btn: DynAnimObj,
    pub minimize_btn: DynAnimObj,
    pub maximize_btn: DynAnimObj,
    pub title_text: DynAnimObj,
}

impl Terminal {
    pub fn instantiate(&self) -> AnimOperation {
        self.layout_result.instantiate()
    }

    pub fn transform(&self) -> &Transform {
        self.background.transform()
    }

    pub fn position_to(&self, to: Vec2, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().position_to(to, time, curve)
    }

    pub fn scale_to(&self, to: Vec2, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().scale_to(to, time, curve)
    }

    pub fn rotate_to(&self, to: f32, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().rotate_to(to, time, curve)
    }

    pub fn position_to_object(
        &self,
        target: &DynAnimObj,
        offset: Vec2,
        time: f32,
        curve: Ease,
    ) -> AnimOperation {
        self.background
            .transform()
            .position_to_object(target, offset, time, curve)
    }
}

/// The text buffer that renders terminal content into a mesh.
#[derive(Clone, Debug)]
pub struct TerminalTextBuffer {
    pub id: Uuid,
    pub transform: Transform,
    pub prompt: String,
    pub font_family: String,
    pub font_size: f32,
    pub default_color: Color,
    pub ansi_colors: [Color; 16],
    pub base16: [Color; 16],
    pub width: f32,
    pub height: f32,
    pub entries: Vec<TerminalEntry>,
    pub current_entry: usize,
    pub dirty: bool,
    cached_size: Option<Vec2>,
}

#[derive(Clone, Debug)]
pub struct TerminalEntry {
    pub command: String,
    pub display_override: Option<String>,
    pub output: String,
    pub prompt: String,
    pub input_reveal: usize,
    pub output_skip: usize,
    pub output_reveal: usize,
    pub pushed_text: Option<String>,
}

impl TerminalTextBuffer {
    pub fn new(
        id: Uuid,
        prompt: String,
        font_family: String,
        font_size: f32,
        width: f32,
        height: f32,
        default_color: Color,
        ansi_colors: [Color; 16],
        base16: [Color; 16],
    ) -> Self {
        Self {
            id,
            transform: Transform::with_uuid(id, Vec3::ZERO),
            prompt,
            font_family,
            font_size,
            default_color,
            ansi_colors,
            base16,
            width,
            height,
            entries: Vec::new(),
            current_entry: 0,
            dirty: true,
            cached_size: None,
        }
    }

    pub fn add_entry(&mut self, command: String, display_override: Option<String>, output: String, prompt: String) {
        self.entries.push(TerminalEntry {
            command,
            display_override,
            output,
            prompt,
            input_reveal: 0,
            output_skip: 0,
            output_reveal: 0,
            pushed_text: None,
        });
        self.current_entry = self.entries.len().saturating_sub(1);
        self.dirty = true;
    }

    pub(crate) fn current_entry_mut(&mut self) -> Option<&mut TerminalEntry> {
        self.entries.get_mut(self.current_entry)
    }

    pub(crate) fn current_entry(&self) -> Option<&TerminalEntry> {
        self.entries.get(self.current_entry)
    }

    fn build_content(&self) -> Vec<ColoredLine> {
        let mut lines: Vec<ColoredLine> = Vec::new();
        let cursor_color = self.base16[5];
        let cmd_color0 = self.base16[12]; // base0C
        let cmd_color1 = self.base16[10]; // base0A

        for entry in &self.entries {
            let display_cmd = entry
                .display_override
                .as_deref()
                .unwrap_or(&entry.command);
            let reveal_len = entry.input_reveal.min(display_cmd.len());
            let visible_cmd = &display_cmd[..reveal_len];

            let prompt_text = if entry.prompt.is_empty() {
                self.prompt.clone()
            } else {
                entry.prompt.clone()
            };
            let prompt_spans = ansi::parse_ansi(&prompt_text, self.default_color, &self.ansi_colors);
            let mut cmd_line = ColoredLine {
                spans: prompt_spans
                    .into_iter()
                    .map(|s| ColoredSpan {
                        color: s.color,
                        text: s.text,
                    })
                    .collect(),
            };
            if !visible_cmd.is_empty() {
                cmd_line.spans.extend(colorize_cmd(visible_cmd, cmd_color0, cmd_color1));
            }
            if reveal_len < display_cmd.len() {
                cmd_line.spans.push(ColoredSpan {
                    color: cursor_color,
                    text: "\u{2588}".to_string(),
                });
            }
            lines.push(cmd_line);

            let mut output_text = entry.output.clone();
            if let Some(ref pushed) = entry.pushed_text {
                output_text.push_str(pushed);
            }

            let skip = entry.output_skip.min(output_text.len());
            let available = output_text.len() - skip;
            let reveal = entry.output_reveal.min(available);

            let all_spans = ansi::parse_ansi(&output_text, self.default_color, &self.ansi_colors);
            let sliced = slice_spans_by_byte_range(&all_spans, skip, reveal);
            if !sliced.is_empty() {
                lines.push(ColoredLine {
                    spans: sliced
                        .into_iter()
                        .map(|s| ColoredSpan {
                            color: s.color,
                            text: s.text,
                        })
                        .collect(),
                });
            }
        }

        lines
    }

    pub fn generate_terminal_mesh(
        &mut self,
        manager: &mut TextManager,
    ) -> Result<(Vec<Vertex>, Vec<Index>, PipelineKind)> {
        use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};

        let content = self.build_content();

        if content.is_empty() {
            self.cached_size = Some(Vec2::ZERO);
            return Ok((vec![], vec![], PipelineKind::Text));
        }

        let line_height = self.font_size * 1.2;

        let mut full_text = String::new();
        let mut color_map: Vec<(std::ops::Range<usize>, Color)> = Vec::new();

        for line in &content {
            for span in &line.spans {
                let span_start = full_text.len();
                full_text.push_str(&span.text);
                color_map.push((span_start..full_text.len(), span.color));
            }
            full_text.push('\n');
        }

        if full_text.is_empty() {
            self.cached_size = Some(Vec2::ZERO);
            return Ok((vec![], vec![], PipelineKind::Text));
        }

        // Byte offset in full_text of each text line (cosmic_text BufferLine).
        // LayoutGlyph.start/.end are relative to their BufferLine's text, not full_text.
        let mut line_start_offsets = vec![0usize];
        for (byte_idx, ch) in full_text.char_indices() {
            if ch == '\n' {
                line_start_offsets.push(byte_idx + 1);
            }
        }

        let font_system = &mut manager.font_system;
        let mut buffer = Buffer::new(font_system, Metrics::new(self.font_size, line_height));

        buffer.set_size(Some(self.width.max(1.0)), Some(self.height.max(1.0)));
        let attrs = Attrs::new().family(Family::Name(&self.font_family));
        buffer.set_text(&full_text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        let scale = manager.scale;
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<Index> = Vec::new();

        for run in buffer.layout_runs() {
            let line_offset = *line_start_offsets.get(run.line_i).unwrap_or(&0);
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), scale);

                let glyph_info = manager
                    .atlas
                    .get_or_insert(&mut manager.font_system, physical.cache_key);

                let x = glyph.x + glyph_info.bearing.x / scale;
                let y = run.line_y - glyph_info.bearing.y / scale;
                let w = glyph_info.width / scale;
                let h = glyph_info.height / scale;

                if h <= 0.0 {
                    continue;
                }

                // Glyph start/end are relative to the BufferLine text
                let glyph_global_start = line_offset + glyph.start;
                let glyph_global_end = line_offset + glyph.end;
                let mut color = self.default_color;
                for (range, span_color) in &color_map {
                    if glyph_global_start >= range.start && glyph_global_end <= range.end {
                        color = *span_color;
                        break;
                    }
                }

                let base = vertices.len() as u32;

                let glyph_color = if glyph_info.is_color {
                    Color::WHITE
                } else {
                    color
                };

                vertices.extend([
                    Vertex {
                        position: vec2(x, y),
                        color: glyph_color,
                        uv: glyph_info.uv_min,
                    },
                    Vertex {
                        position: vec2(x + w, y),
                        color: glyph_color,
                        uv: vec2(glyph_info.uv_max.x, glyph_info.uv_min.y),
                    },
                    Vertex {
                        position: vec2(x + w, y + h),
                        color: glyph_color,
                        uv: glyph_info.uv_max,
                    },
                    Vertex {
                        position: vec2(x, y + h),
                        color: glyph_color,
                        uv: vec2(glyph_info.uv_min.x, glyph_info.uv_max.y),
                    },
                ]);

                indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
            }
        }

        if !vertices.is_empty() {
            for v in &mut vertices {
                v.position.x += -self.width / 2.0 + 12.0;
                v.position.y += -self.height / 2.0 + 12.0;
            }
            self.cached_size = Some(vec2(self.width, self.height));
        } else {
            self.cached_size = None;
        }

        Ok((vertices, indices, PipelineKind::Text))
    }
}

/// Given parsed ANSI spans and a raw byte range [skip, skip+reveal) in the
/// original text, return spans covering only that range. Spans that fall
/// entirely outside the range are omitted; partial spans are sliced at their
/// byte boundaries (on the assumption the slice positions are ASCII-safe).
fn slice_spans_by_byte_range(spans: &[ansi::AnsiSpan], skip: usize, reveal: usize) -> Vec<ansi::AnsiSpan> {
    if reveal == 0 {
        return Vec::new();
    }
    let end = skip + reveal;
    let mut result = Vec::new();

    for span in spans {
        if span.byte_end > skip && span.byte_start < end {
            let lo = span.byte_start.max(skip) - span.byte_start;
            let hi = span.byte_end.min(end) - span.byte_start;
            if lo < hi {
                result.push(ansi::AnsiSpan {
                    color: span.color,
                    text: span.text[lo..hi].to_string(),
                    byte_start: span.byte_start + lo,
                    byte_end: span.byte_start + hi,
                });
            }
        }
    }

    result
}

fn colorize_cmd(cmd: &str, cmd_color0: Color, cmd_color1: Color) -> Vec<ColoredSpan> {
    let mut spans = Vec::new();
    let mut first = true;
    let mut pos = 0;
    let bytes = cmd.as_bytes();

    while pos < bytes.len() {
        let ws_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos > ws_start {
            spans.push(ColoredSpan {
                color: cmd_color0,
                text: cmd[ws_start..pos].to_string(),
            });
        }
        if pos >= bytes.len() {
            break;
        }
        let tok_start = pos;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        let color = if first {
            first = false;
            cmd_color0
        } else {
            cmd_color1
        };
        spans.push(ColoredSpan {
            color,
            text: cmd[tok_start..pos].to_string(),
        });
    }
    spans
}

impl AnimObjectTrait for TerminalTextBuffer {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn uuid(&self) -> Uuid {
        self.id
    }
    fn size(&self) -> Vec2 {
        if let Some(cached) = self.cached_size {
            return cached;
        }
        vec2(self.width, self.height)
    }
    fn generate_mesh(&mut self, mgr: &mut TextManager) -> MeshResult {
        self.generate_terminal_mesh(mgr)
            .map_err(|e| anyhow::anyhow!("terminal mesh generation: {e}"))
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

#[derive(Clone, Debug)]
struct ColoredLine {
    spans: Vec<ColoredSpan>,
}

#[derive(Clone, Debug)]
struct ColoredSpan {
    color: Color,
    text: String,
}

/// Factory function to create a Terminal from core data.
pub fn terminal(
    position: Vec3,
    shell: String,
    prompt: String,
    font_family: String,
    font_size: f32,
    width: f32,
    height: f32,
    background_color: Color,
    text_color: Color,
    _term_id: Uuid,
    text_buffer_id: Uuid,
    bg_id: Uuid,
    container_id: Uuid,
    close_btn_id: Uuid,
    minimize_btn_id: Uuid,
    maximize_btn_id: Uuid,
    title_id: Uuid,
    title_bar_bg_id: Uuid,
    title: String,
    title_font_size: f32,
    theme: &Theme,
) -> Terminal {
    let _ = shell; // shell is used at animation definition time (in TerminalInputBuilder)
    let circle_r = 12.0;

    let b = &theme.base.colors;
    let close_btn = circle(
        Transform::with_uuid(close_btn_id, Vec3::ZERO),
        circle_r,
        b[8], // base08
    );
    let minimize_btn = circle(
        Transform::with_uuid(minimize_btn_id, Vec3::ZERO),
        circle_r,
        b[9], // base09
    );
    let maximize_btn = circle(
        Transform::with_uuid(maximize_btn_id, Vec3::ZERO),
        circle_r,
        b[11], // base0B
    );
    let title_text = text_fn(
        Transform::with_uuid(title_id, Vec3::ZERO),
        title,
        "sans-serif".to_string(),
        Align::Left,
        b[5], // base05
        title_font_size,
    );

    let title_layout = layout_with_ids(
        Vec3::ZERO,
        PinPoint::C,
        vec![
            LayoutItem::Object(close_btn.clone()),
            LayoutItem::Object(minimize_btn.clone()),
            LayoutItem::Object(maximize_btn.clone()),
            LayoutItem::Object(title_text.clone()),
        ],
        LayoutBackground {
            color: b[1], // base01
            corner_radius: 5.,
        },
        LayoutDir::Row,
        LayoutAlignment::Center,
        8.0,
        -35.0,
        -35.0,
        25.0,
        25.0,
        0.0,
        0.0,
        Some(title_bar_bg_id),
        None,
    );

    let ansi_colors = ansi::ansi_table_from_base16(&theme.base);
    let text_buffer = TerminalTextBuffer::new(
        text_buffer_id,
        prompt,
        font_family,
        font_size,
        width,
        height,
        text_color,
        ansi_colors,
        theme.base.colors,
    );

    let layout_result = layout_with_ids(
        position,
        PinPoint::C,
        vec![
            LayoutItem::Layout(title_layout),
            LayoutItem::Object(DynAnimObj(Box::new(text_buffer.clone()))),
        ],
        LayoutBackground {
            color: background_color,
            corner_radius: 5.,
        },
        LayoutDir::Column,
        LayoutAlignment::Start,
        25.0,
        0.0,
        0.0,
        0.0,
        0.0,
        width,
        height,
        Some(bg_id),
        Some(container_id),
    );

    Terminal {
        background: layout_result.background.clone(),
        text_buffer,
        layout_result,
        close_btn,
        minimize_btn,
        maximize_btn,
        title_text,
    }
}
