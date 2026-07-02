use glam::vec2;
use uuid::Uuid;

use crate::{
    anim_object::{
        render::PipelineKind,
        text::{TextManager, code::Code},
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

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((0.0, 0.0), 1.0);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing.x;
            let y = run.line_y - glyph_info.bearing.y;

            let w = glyph_info.width;
            let h = glyph_info.height;

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

    (vertices, indices, PipelineKind::Text)
}
