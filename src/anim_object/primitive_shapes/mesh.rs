use crate::{
    anim_object::{primitive_shapes::Square, render::PipelineKind},
    renderer::{Index, Vertex},
};

pub fn generate_square_mesh_data(square: &Square) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let (w, h) = square.size;

    let hw = w * 0.5;
    let hh = h * 0.5;

    let color = [square.color.0, square.color.1, square.color.2];

    let vertices = vec![
        Vertex {
            position: [-hw, -hh],
            color,
            uv: [0., 0.],
        },
        Vertex {
            position: [hw, -hh],
            color,
            uv: [1., 0.],
        },
        Vertex {
            position: [hw, hh],
            color,
            uv: [1., 1.],
        },
        Vertex {
            position: [-hw, hh],
            color,
            uv: [0., 1.],
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices, PipelineKind::Shape)
}
