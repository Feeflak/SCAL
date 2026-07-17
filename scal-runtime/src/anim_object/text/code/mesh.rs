use std::collections::HashSet;

use anyhow::{Context, Result};
use glam::vec2;
use scal_core::{CodeAnimationStyle, Color};
use uuid::Uuid;

use crate::{
    anim_object::{
        render::PipelineKind,
        text::TextManager,
        text::code::Code,
    },
    renderer::Vertex,
};

pub fn generate_code_mesh(
    manager: &mut TextManager,
    id: Uuid,
    code: &mut Code,
) -> Result<(Vec<Vertex>, Vec<u32>, PipelineKind)> {
    code.update_highlight_if_dirty(&mut manager.code_highlighter)
        .context("code highlighting did not succeed")?;
    let buffer = manager.layout_code(code, id)?;
    let scale = manager.scale;

    let reveal = code.anim_reveal;
    let spacing = code.anim_spacing;
    let line_start = code.anim_line_start;
    let line_end = code.anim_line_end;

    let line_height = code.font_size * 1.2;
    let new_line_count = line_end.saturating_sub(line_start);

    let num_animated_lines = new_line_count;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let layout_runs: Vec<_> = buffer.layout_runs().collect();

    let total_animated_glyphs = if matches!(
        code.anim_style,
        CodeAnimationStyle::TypeWriter | CodeAnimationStyle::TypeWriterInstantResize
    ) {
        let mut gl = 0usize;
        let mut li = 0usize;
        let mut ply: Option<f32> = None;
        for run in &layout_runs {
            let cur_y = run.line_y;
            if let Some(prev) = ply {
                if (cur_y - prev).abs() > line_height * 0.5 {
                    li += 1;
                }
            }
            ply = Some(cur_y);
            if li >= line_start && li < line_end {
                gl += run.glyphs.len();
            }
        }
        gl
    } else {
        0
    };
    let mut animated_glyphs_emitted = 0usize;

    let highlighted_lines: HashSet<usize> = code.highlighted_lines();
    let overall_progress = code.max_highlight_progress();
    let has_highlights = !highlighted_lines.is_empty();

    let mut li: usize = 0;
    let mut ply: Option<f32> = None;
    for run in &layout_runs {
        let cur_y = run.line_y;
        if let Some(prev) = ply {
            if (cur_y - prev).abs() > line_height * 0.5 {
                li += 1;
            }
        }
        ply = Some(cur_y);

        let after_animated = li >= line_end;
        let in_animated = li >= line_start && li < line_end;

        let y_offset = if in_animated {
            (li - line_start) as f32 * line_height * (spacing - 1.0)
        } else if after_animated && new_line_count > 0 {
            -(new_line_count as f32 - 1.0) * line_height * (1.0 - spacing)
        } else {
            0.0
        };

        let line_vis = if in_animated {
            let line_index = li - line_start;
            match code.anim_style {
                CodeAnimationStyle::TypeWriter | CodeAnimationStyle::TypeWriterInstantResize => {
                    true
                }
                CodeAnimationStyle::Fold => {
                    let line_threshold = reveal * num_animated_lines as f32;
                    (line_index as f32) < line_threshold
                }
                CodeAnimationStyle::Reveal => {
                    // Lines always visible — alpha is controlled per-glyph
                    true
                }
            }
        } else {
            true
        };

        if !line_vis {
            continue;
        }

        for glyph in run.glyphs {
            if in_animated
                && matches!(
                    code.anim_style,
                    CodeAnimationStyle::TypeWriter | CodeAnimationStyle::TypeWriterInstantResize
                )
            {
                let threshold = total_animated_glyphs as f32 * reveal;
                if animated_glyphs_emitted as f32 >= threshold {
                    break;
                }
                animated_glyphs_emitted += 1;
            }

            let physical = glyph.physical((0.0, 0.0), scale);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing.x / scale;
            let y = run.line_y - glyph_info.bearing.y / scale + y_offset;

            let w = glyph_info.width / scale;
            let h = glyph_info.height / scale;

            if h <= 0.0 {
                continue;
            }

            let base = vertices.len() as u32;

            if glyph_info.is_color {
                let alpha = if in_animated && code.anim_style == CodeAnimationStyle::Reveal {
                    reveal
                } else {
                    1.0
                };
                let color = Color::new(1.0, 1.0, 1.0, alpha);
                vertices.extend([
                    Vertex { position: vec2(x, y), color, uv: glyph_info.uv_min },
                    Vertex { position: vec2(x + w, y), color, uv: vec2(glyph_info.uv_max.x, glyph_info.uv_min.y) },
                    Vertex { position: vec2(x + w, y + h), color, uv: glyph_info.uv_max },
                    Vertex { position: vec2(x, y + h), color, uv: vec2(glyph_info.uv_min.x, glyph_info.uv_max.y) },
                ]);
                indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
                continue;
            }

            let mut color = if let Some(glyph_color) = glyph.color_opt {
                Color::new(
                    glyph_color.r() as f32 / 255.0,
                    glyph_color.g() as f32 / 255.0,
                    glyph_color.b() as f32 / 255.0,
                    glyph_color.a() as f32 / 255.0,
                )
            } else {
                Color::WHITE
            };

            if has_highlights && !highlighted_lines.contains(&li) {
                let darken = 1.0 - overall_progress * 0.5;
                color = Color::new(
                    color.r * darken,
                    color.g * darken,
                    color.b * darken,
                    color.a,
                );
            }

            if in_animated && code.anim_style == CodeAnimationStyle::Reveal {
                color = Color::new(color.r, color.g, color.b, color.a * reveal);
            }

            vertices.extend([
                Vertex {
                    position: vec2(x, y),
                    color,
                    uv: glyph_info.uv_min,
                },
                Vertex {
                    position: vec2(x + w, y),
                    color,
                    uv: vec2(glyph_info.uv_max.x, glyph_info.uv_min.y),
                },
                Vertex {
                    position: vec2(x + w, y + h),
                    color,
                    uv: glyph_info.uv_max,
                },
                Vertex {
                    position: vec2(x, y + h),
                    color,
                    uv: vec2(glyph_info.uv_min.x, glyph_info.uv_max.y),
                },
            ]);

            indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }

    if !vertices.is_empty() {
        let mut min = vertices[0].position;
        let mut max = min;
        for v in &vertices {
            min = min.min(v.position);
            max = max.max(v.position);
        }
        let center = (min + max) * 0.5;
        for v in &mut vertices {
            v.position -= center;
        }
        code.cached_size = Some(max - min);
    } else {
        code.cached_size = None;
    }

    let run_count = buffer.layout_runs().count();
    log::debug!(
        "generate_code_mesh style={:?} reveal={:.4} spacing={:.4} line=[{}, {}) animated_glyphs={} emitted={} verts={} indices={} runs={}",
        code.anim_style,
        reveal,
        spacing,
        line_start,
        line_end,
        total_animated_glyphs,
        animated_glyphs_emitted,
        vertices.len(),
        indices.len(),
        run_count
    );

    Ok((vertices, indices, PipelineKind::Text))
}


