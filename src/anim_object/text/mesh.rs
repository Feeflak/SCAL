use crate::{
    anim_object::{
        render::PipelineKind,
        text::{Text, TextManager},
    },
    renderer::Vertex,
};
pub fn generate_text_mesh(
    manager: &mut TextManager,
    text: &Text,
) -> (Vec<Vertex>, Vec<u32>, PipelineKind) {
    let buffer = manager.layout(text);

    let mut vertices = vec![];
    let mut indices = vec![];

    let color = [text.color.0, text.color.1, text.color.2];

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((0., 0.), 1.0);

            let glyph_info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);

            let x = glyph.x + glyph_info.bearing[0];

            let y = run.line_y + glyph_info.bearing[1];

            let w = glyph_info.width;

            let h = glyph_info.height;

            let base = vertices.len() as u32;

            vertices.extend([
                Vertex {
                    position: [x, y],
                    color,
                    uv: glyph_info.uv_min,
                },
                Vertex {
                    position: [x + w, y],
                    color,
                    uv: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                },
                Vertex {
                    position: [x + w, y + h],
                    color,
                    uv: glyph_info.uv_max,
                },
                Vertex {
                    position: [x, y + h],
                    color,
                    uv: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                },
            ]);

            indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }

    (vertices, indices, PipelineKind::Text)
}
