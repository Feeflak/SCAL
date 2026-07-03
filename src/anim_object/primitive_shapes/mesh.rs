use crate::{
    anim_object::{primitive_shapes::Rectangle, render::PipelineKind},
    renderer::{Index, Vertex},
};
use glam::vec2;

pub fn generate_rectangle_mesh_data(
    rectangle: &Rectangle,
) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let size = rectangle.size * 0.5;
    let color = rectangle.color;

    let vertices = vec![
        Vertex {
            position: -size,
            color,
            uv: vec2(0., 0.),
        },
        Vertex {
            position: vec2(size.x, -size.y),
            color,
            uv: vec2(1., 0.),
        },
        Vertex {
            position: size,
            color,
            uv: vec2(1., 1.),
        },
        Vertex {
            position: vec2(-size.x, size.y),
            color,
            uv: vec2(0., 1.),
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices, PipelineKind::Shape)
}
