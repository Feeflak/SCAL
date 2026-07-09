use glam::vec2;
use uuid::Uuid;

use crate::{
    anim_object::{
        render::PipelineKind,
        text::TextManager,
        text::code::{Code, CodeAnimationStyle, CodeHighlightKind},
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

    let prefix_len = if code.show_line_numbers {
        let line_count = code.lines.len();
        let gutter_digits = if line_count > 0 {
            (line_count.ilog10() + 1) as usize
        } else {
            1
        };
        gutter_digits + 1
    } else {
        0
    };

    let source_line_offsets: Vec<usize> = {
        let mut offsets = Vec::new();
        let mut offset = 0usize;
        for line in code.source_code.lines() {
            offsets.push(offset);
            offset += line.len() + 1;
        }
        offsets
    };

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

    let need_highlights = !code.highlights.is_empty();
    let mut line_ys: Vec<f32> = Vec::new();
    let mut line_x_mins: Vec<f32> = Vec::new();
    let mut line_x_maxs: Vec<f32> = Vec::new();
    let mut line_glyph_info: Vec<Vec<(f32, f32, usize, usize)>> = Vec::new();
    let mut line_visible: Vec<bool> = Vec::new();

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
            }
        } else {
            true
        };

        if need_highlights {
            while line_ys.len() <= li {
                line_ys.push(0.0);
                line_x_mins.push(f32::MAX);
                line_x_maxs.push(f32::MIN);
                line_glyph_info.push(Vec::new());
                line_visible.push(false);
            }
            line_ys[li] = cur_y + y_offset;
            line_visible[li] = line_vis;
        }

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

            if need_highlights && li < line_glyph_info.len() {
                line_glyph_info[li].push((x, x + w, glyph.start, glyph.end));
                line_x_mins[li] = line_x_mins[li].min(x);
                line_x_maxs[li] = line_x_maxs[li].max(x + w);
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

    if need_highlights && !code.highlights.is_empty() {
        for highlight in &code.highlights {
            let mut hl_color = highlight.color;
            hl_color.a *= highlight.progress;
            match &highlight.kind {
                CodeHighlightKind::Lines { ranges } => {
                    for range in ranges {
                        for hl_li in range.clone() {
                            if hl_li >= line_ys.len() || !line_visible[hl_li] {
                                continue;
                            }
                            let y_center = line_ys[hl_li];
                            let top = y_center - line_height;
                            let bottom = y_center + line_height * 0.15;
                            let left = if line_x_mins[hl_li] <= line_x_maxs[hl_li] {
                                line_x_mins[hl_li]
                            } else {
                                0.0
                            };
                            let right = if line_x_maxs[hl_li] >= line_x_mins[hl_li] {
                                line_x_maxs[hl_li]
                            } else {
                                code.font_size * 2.0
                            };
                            emit_quad(
                                &mut vertices,
                                &mut indices,
                                left,
                                bottom,
                                right,
                                top,
                                hl_color,
                            );
                        }
                    }
                }
                CodeHighlightKind::Pattern { regex } => {
                    if regex.is_empty() {
                        continue;
                    }
                    let Ok(re) = regex::Regex::new(regex) else {
                        continue;
                    };
                    let src_matches: Vec<(usize, usize)> = re
                        .find_iter(&code.source_code)
                        .map(|m| (m.start(), m.end()))
                        .collect();

                    for &(match_start_byte, match_end_byte) in &src_matches {
                        let hl_li = match source_line_offsets.binary_search(&match_start_byte) {
                            Ok(idx) => idx,
                            Err(idx) => {
                                if idx == 0 {
                                    continue;
                                }
                                idx - 1
                            }
                        };

                        if hl_li >= line_ys.len() || !line_visible[hl_li] {
                            continue;
                        }

                        let buf_start = match_start_byte + (hl_li + 1) * prefix_len;
                        let buf_end = match_end_byte + (hl_li + 1) * prefix_len;

                        let mut hl_min_x = f32::MAX;
                        let mut hl_max_x = f32::MIN;
                        let mut found = false;

                        if hl_li < line_glyph_info.len() {
                            for &(gx, gx_end, g_start, g_end) in &line_glyph_info[hl_li] {
                                if g_start >= buf_start && g_end <= buf_end {
                                    hl_min_x = hl_min_x.min(gx);
                                    hl_max_x = hl_max_x.max(gx_end);
                                    found = true;
                                }
                            }
                        }

                        if found {
                            let y_center = line_ys[hl_li];
                            let top = y_center - line_height;
                            let bottom = y_center + line_height * 0.15;
                            emit_quad(
                                &mut vertices,
                                &mut indices,
                                hl_min_x,
                                bottom,
                                hl_max_x,
                                top,
                                hl_color,
                            );
                        }
                    }
                }
            }
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

    (vertices, indices, PipelineKind::Text)
}

fn emit_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
) {
    let base = vertices.len() as u32;
    let sentinel_uv = glam::vec2(-1.0, -1.0);
    vertices.extend([
        Vertex {
            position: vec2(x1, y1),
            color,
            uv: sentinel_uv,
        },
        Vertex {
            position: vec2(x2, y1),
            color,
            uv: sentinel_uv,
        },
        Vertex {
            position: vec2(x2, y2),
            color,
            uv: sentinel_uv,
        },
        Vertex {
            position: vec2(x1, y2),
            color,
            uv: sentinel_uv,
        },
    ]);
    indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
}
