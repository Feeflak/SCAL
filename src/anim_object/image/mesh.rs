use crate::{
    anim_object::{image::*, render::PipelineKind},
    renderer::{Index, Vertex},
};
use glam::vec2;

pub fn generate_image_mesh_data(image: &Image) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let quad_size = match image.stretch {
        StretchMode::Fill => image.size,
        StretchMode::Fit => {
            match image::ImageReader::open(&image.path)
                .ok()
                .and_then(|r| r.into_dimensions().ok())
            {
                Some((tex_w, tex_h)) => {
                    let img_aspect = tex_w as f32 / tex_h as f32;
                    let quad_aspect = image.size.x / image.size.y;
                    if img_aspect > quad_aspect {
                        vec2(image.size.x, image.size.x / img_aspect)
                    } else {
                        vec2(image.size.y * img_aspect, image.size.y)
                    }
                }
                None => image.size,
            }
        }
    };
    let half = quad_size * 0.5;
    let uvs = [vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0)];
    let vertices = vec![
        Vertex { position: -half,             color: image.color, uv: uvs[0] },
        Vertex { position: vec2(half.x, -half.y), color: image.color, uv: uvs[1] },
        Vertex { position: half,              color: image.color, uv: uvs[2] },
        Vertex { position: vec2(-half.x, half.y), color: image.color, uv: uvs[3] },
    ];
    let indices = vec![0, 1, 2, 2, 3, 0];
    (vertices, indices, PipelineKind::Image)
}
