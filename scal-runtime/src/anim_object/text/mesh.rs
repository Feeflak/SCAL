use scal_core::{Color, ModificationType, TextModifier as CoreTextModifier};

use crate::{
    anim_object::{
        render::PipelineKind,
        text::{Text, TextManager},
    },
    renderer::Vertex,
};
use glam::{vec2, Vec2};

struct GlyphQuad {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    uv_min: Vec2,
    uv_max: Vec2,
    is_color: bool,
}

pub fn generate_text_mesh(
    manager: &mut TextManager,
    text: &mut Text,
) -> (Vec<Vertex>, Vec<u32>, PipelineKind) {
    let buffer = manager.layout(text);
    let scale = manager.scale;

    let mut quads: Vec<GlyphQuad> = vec![];

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((0., 0.), scale);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing.x / scale;
            let y = run.line_y - glyph_info.bearing.y / scale;
            let w = glyph_info.width / scale;
            let h = glyph_info.height / scale;

            quads.push(GlyphQuad {
                x,
                y,
                w,
                h,
                uv_min: glyph_info.uv_min,
                uv_max: glyph_info.uv_max,
                is_color: glyph_info.is_color,
            });
        }
    }

    if quads.is_empty() {
        text.cached_size = None;
        return (vec![], vec![], PipelineKind::Text);
    }

    let mut min = quads[0].x;
    let mut max = quads[0].x + quads[0].w;
    let mut min_y = quads[0].y;
    let mut max_y = quads[0].y + quads[0].h;
    for q in &quads {
        min = min.min(q.x);
        max = max.max(q.x + q.w);
        min_y = min_y.min(q.y);
        max_y = max_y.max(q.y + q.h);
    }
    let center = vec2((min + max) * 0.5, (min_y + max_y) * 0.5);

    let mut vertices = vec![];
    let mut indices = vec![];

    for modifier in &text.modifications {
        emit_modifier_quads(&modifier, &quads, center, &mut vertices, &mut indices);
    }

    emit_base_quads(text.color, &quads, center, &mut vertices, &mut indices);

    text.cached_size = Some(vec2(max - min, max_y - min_y));

    (vertices, indices, PipelineKind::Text)
}

fn emit_quad(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: Color,
    uv_min: Vec2,
    uv_max: Vec2,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;

    vertices.extend([
        Vertex {
            position: vec2(x, y),
            color,
            uv: uv_min,
        },
        Vertex {
            position: vec2(x + w, y),
            color,
            uv: vec2(uv_max.x, uv_min.y),
        },
        Vertex {
            position: vec2(x + w, y + h),
            color,
            uv: uv_max,
        },
        Vertex {
            position: vec2(x, y + h),
            color,
            uv: vec2(uv_min.x, uv_max.y),
        },
    ]);

    indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn emit_base_quads(
    text_color: Color,
    quads: &[GlyphQuad],
    center: Vec2,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    for q in quads {
        let color = if q.is_color { Color::WHITE } else { text_color };
        emit_quad(
            q.x - center.x,
            q.y - center.y,
            q.w,
            q.h,
            color,
            q.uv_min,
            q.uv_max,
            vertices,
            indices,
        );
    }
}

fn emit_modifier_quads(
    modifier: &CoreTextModifier,
    quads: &[GlyphQuad],
    center: Vec2,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let rot_rad = modifier.rotation.to_radians();
    let cos = rot_rad.cos();
    let sin = rot_rad.sin();

    for q in quads {
        let color = if q.is_color { Color::WHITE } else { modifier.color };

        let (qx, qy, qw, qh) = match modifier.modification_type {
            ModificationType::Outline => {
                let s = modifier.softness;
                (q.x - s, q.y - s, q.w + 2.0 * s, q.h + 2.0 * s)
            }
            ModificationType::Infill => {
                let s = modifier.softness;
                (q.x + s, q.y + s, (q.w - 2.0 * s).max(0.1), (q.h - 2.0 * s).max(0.1))
            }
        };

        let mut cx = qx + qw * 0.5;
        let mut cy = qy + qh * 0.5;

        let dx = cx - center.x;
        let dy = cy - center.y;
        let sx = dx * modifier.scale.x;
        let sy = dy * modifier.scale.y;
        let rx = sx * cos - sy * sin;
        let ry = sx * sin + sy * cos;
        cx = center.x + rx;
        cy = center.y + ry;
        let final_w = qw * modifier.scale.x;
        let final_h = qh * modifier.scale.y;

        let x0 = cx - final_w * 0.5 + modifier.pos_offset.x;
        let y0 = cy - final_h * 0.5 + modifier.pos_offset.y;

        emit_quad(
            x0 - center.x,
            y0 - center.y,
            final_w,
            final_h,
            color,
            q.uv_min,
            q.uv_max,
            vertices,
            indices,
        );
    }
}
