use glam::Vec2;

use crate::anim_object::image::StretchMode;
use crate::anim_object::render::PipelineKind;
use crate::renderer::{Index, Vertex};

use super::Svg;

pub fn generate_svg_mesh_data(svg: &Svg) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let quad_size = match svg.stretch {
        StretchMode::Fill => svg.size,
        StretchMode::Fit => {
            let mut fallback = true;
            let mut result = svg.size;
            if let Ok(data) = std::fs::read(&svg.path) {
                if let Ok(tree) = usvg::Tree::from_data(&data, &usvg::Options::default()) {
                    let sz = tree.size();
                    let img_aspect = sz.width() as f32 / sz.height() as f32;
                    let quad_aspect = svg.size.x / svg.size.y;
                    result = if img_aspect > quad_aspect {
                        Vec2::new(svg.size.x, svg.size.x / img_aspect)
                    } else {
                        Vec2::new(svg.size.y * img_aspect, svg.size.y)
                    };
                    fallback = false;
                }
            }
            if fallback { svg.size } else { result }
        }
    };
    let half = quad_size * 0.5;
    let uvs = [
        glam::vec2(0.0, 0.0),
        glam::vec2(1.0, 0.0),
        glam::vec2(1.0, 1.0),
        glam::vec2(0.0, 1.0),
    ];
    let vertices = vec![
        Vertex { position: -half,                    color: svg.tint, uv: uvs[0] },
        Vertex { position: glam::vec2(half.x, -half.y), color: svg.tint, uv: uvs[1] },
        Vertex { position: half,                     color: svg.tint, uv: uvs[2] },
        Vertex { position: glam::vec2(-half.x, half.y), color: svg.tint, uv: uvs[3] },
    ];
    let indices = vec![0, 1, 2, 2, 3, 0];
    (vertices, indices, PipelineKind::Image)
}
