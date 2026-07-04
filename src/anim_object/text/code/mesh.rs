use glam::vec2;
use uuid::Uuid;

use crate::{
    anim_object::{
        render::PipelineKind,
        text::code::{Code, CodeAnimationStyle},
        text::TextManager,
    },
    renderer::Vertex,
    types::Color,
};

pub fn generate_code_mesh(
    manager: &mut TextManager,
    id: Uuid,
    code: &mut Code,
) -> (Vec<Vertex>, Vec<u32>, PipelineKind) {
    code.update_highlight_if_dirty(&mut manager.code_highlighter)
        .expect("code highlighting did not succeed");
    let buffer = manager.layout_code(code, id);

    let reveal = code.anim_reveal;
    let spacing = code.anim_spacing;
    let line_start = code.anim_line_start;
    let line_end = code.anim_line_end;

    let line_height = code.font_size * 1.2;
    let new_line_count = line_end.saturating_sub(line_start);

    let num_animated_lines = new_line_count;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // TypeWriter: pre-count total glyphs in animated range
    let total_animated_glyphs = if matches!(code.anim_style, CodeAnimationStyle::TypeWriter) {
        buffer
            .layout_runs()
            .enumerate()
            .filter(|(i, _)| *i >= line_start && *i < line_end)
            .flat_map(|(_, r)| r.glyphs.iter())
            .count()
    } else {
        0
    };
    let mut animated_glyphs_emitted = 0usize;

    for (run_idx, run) in buffer.layout_runs().enumerate() {
        let after_animated = run_idx >= line_end;
        let in_animated = run_idx >= line_start && run_idx < line_end;

        let y_offset = if after_animated {
            new_line_count as f32 * line_height * spacing
        } else if in_animated {
            (run_idx - line_start) as f32 * line_height * spacing
        } else {
            0.0
        };

        let line_visible = if in_animated {
            let line_idx = run_idx - line_start;
            match code.anim_style {
                CodeAnimationStyle::TypeWriter => true,
                CodeAnimationStyle::Fold => {
                    let line_threshold = reveal * num_animated_lines as f32;
                    (line_idx as f32) < line_threshold
                }
            }
        } else {
            true
        };

        if !line_visible {
            continue;
        }

        for glyph in run.glyphs {
            if in_animated
                && matches!(code.anim_style, CodeAnimationStyle::TypeWriter)
            {
                let threshold = total_animated_glyphs as f32 * reveal;
                if animated_glyphs_emitted as f32 >= threshold {
                    break;
                }
                animated_glyphs_emitted += 1;
            }

            let physical = glyph.physical((0.0, 0.0), 1.0);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing.x;
            let y = run.line_y - glyph_info.bearing.y + y_offset;

            let w = glyph_info.width;
            let h = glyph_info.height;

            if h <= 0.0 {
                continue;
            }

            let base = vertices.len() as u32;

            let color = if let Some(glyph_color) = glyph.color_opt {
                Color::new(
                    glyph_color.r() as f32 / 255.0,
                    glyph_color.g() as f32 / 255.0,
                    glyph_color.b() as f32 / 255.0,
                    glyph_color.a() as f32 / 255.0,
                )
            } else {
                Color::WHITE
            };

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

    let run_count = buffer.layout_runs().count();
    log::debug!(
        "generate_code_mesh style={:?} reveal={:.4} spacing={:.4} line=[{}, {}) animated_glyphs={} emitted={} verts={} indices={} runs={}",
        code.anim_style, reveal, spacing, line_start, line_end,
        total_animated_glyphs, animated_glyphs_emitted,
        vertices.len(), indices.len(), run_count
    );

    (vertices, indices, PipelineKind::Text)
}
