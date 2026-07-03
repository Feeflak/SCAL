use std::f32::consts::PI;

use crate::{
    anim_object::{primitive_shapes::*, render::PipelineKind},
    renderer::{Index, Vertex},
};
use glam::{vec2, Vec2};

pub fn generate_rectangle_mesh_data(
    rectangle: &Rectangle,
) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let half = rectangle.size * 0.5;
    let r = rectangle
        .corner_radius
        .clamp(0.0, rectangle.size.x.min(rectangle.size.y) * 0.5);
    let per_corner = if r > 0.0 { 8 } else { 1 };
    let total = (per_corner * 4) as usize;

    let corner_centers = [
        vec2(half.x - r, half.y - r),
        vec2(-half.x + r, half.y - r),
        vec2(-half.x + r, -half.y + r),
        vec2(half.x - r, -half.y + r),
    ];

    let mut vertices = Vec::with_capacity(total + 1);
    vertices.push(Vertex {
        position: Vec2::ZERO,
        color: rectangle.color,
        uv: vec2(0.5, 0.5),
    });

    for (ci, &center) in corner_centers.iter().enumerate() {
        let start = ci as f32 * PI / 2.0;
        for s in 0..per_corner {
            let angle = start + s as f32 * (PI / 2.0 / per_corner as f32);
            let pos = center + vec2(r * angle.cos(), r * angle.sin());
            vertices.push(Vertex {
                position: pos,
                color: rectangle.color,
                uv: pos / half * 0.5 + 0.5,
            });
        }
    }

    let mut indices = Vec::with_capacity(total * 3);
    for i in 0..total {
        indices.push(0);
        indices.push(i as u32 + 1);
        indices.push(((i + 1) % total) as u32 + 1);
    }

    (vertices, indices, PipelineKind::Shape)
}

pub fn generate_circle_mesh_data(
    circle: &Circle,
) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let segments: usize = 32;
    let mut vertices = Vec::with_capacity(segments + 1);
    vertices.push(Vertex {
        position: Vec2::ZERO,
        color: circle.color,
        uv: vec2(0.5, 0.5),
    });

    for i in 0..segments {
        let angle = i as f32 * (2.0 * PI / segments as f32);
        let pos = vec2(circle.radius * angle.cos(), circle.radius * angle.sin());
        vertices.push(Vertex {
            position: pos,
            color: circle.color,
            uv: pos / circle.radius * 0.5 + 0.5,
        });
    }

    let mut indices = Vec::with_capacity(segments * 3);
    for i in 0..segments {
        indices.push(0);
        indices.push(i as u32 + 1);
        indices.push(((i + 1) % segments) as u32 + 1);
    }

    (vertices, indices, PipelineKind::Shape)
}

pub fn generate_polygon_mesh_data(
    polygon: &Polygon,
) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let sides = polygon.sides.max(3) as usize;
    let mut vertices = Vec::with_capacity(sides + 1);
    vertices.push(Vertex {
        position: Vec2::ZERO,
        color: polygon.color,
        uv: vec2(0.5, 0.5),
    });

    for i in 0..sides {
        let angle = i as f32 * (2.0 * PI / sides as f32) - PI / 2.0;
        let pos = vec2(polygon.radius * angle.cos(), polygon.radius * angle.sin());
        vertices.push(Vertex {
            position: pos,
            color: polygon.color,
            uv: pos / polygon.radius * 0.5 + 0.5,
        });
    }

    let mut indices = Vec::with_capacity(sides * 3);
    for i in 0..sides {
        indices.push(0);
        indices.push(i as u32 + 1);
        indices.push(((i + 1) % sides) as u32 + 1);
    }

    (vertices, indices, PipelineKind::Shape)
}
