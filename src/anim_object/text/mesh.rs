use crate::{
    anim_object::{
        render::PipelineKind,
        text::{Text, TextManager},
    },
    renderer::Vertex,
    types::Vec2,
};
pub fn generate_text_mesh(
    manager: &mut TextManager,
    text: &Text,
) -> (Vec<Vertex>, Vec<u32>, PipelineKind) {
    let buffer = manager.layout(text);

    let mut vertices = vec![];
    let mut indices = vec![];

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((0., 0.), 1.0);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing.x;
            let y = run.line_y - glyph_info.bearing.y;
            let w = glyph_info.width;
            let h = glyph_info.height;

            let base = vertices.len() as u32;

            vertices.extend([
                Vertex {
                    position: Vec2::new(x, y),
                    color: text.color,
                    uv: glyph_info.uv_min,
                },
                Vertex {
                    position: Vec2::new(x + w, y),
                    color: text.color,
                    uv: Vec2::new(glyph_info.uv_max.x, glyph_info.uv_min.y),
                },
                Vertex {
                    position: Vec2::new(x + w, y + h),
                    color: text.color,
                    uv: glyph_info.uv_max,
                },
                Vertex {
                    position: Vec2::new(x, y + h),
                    color: text.color,
                    uv: Vec2::new(glyph_info.uv_min.x, glyph_info.uv_max.y),
                },
            ]);

            indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }

    (vertices, indices, PipelineKind::Text)
}
