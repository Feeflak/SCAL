//! Mesh of a [`Vector`]: the line as a strip of quads with mitered corners, cut into dashes if
//! asked, and the arrow head as one triangle. Nothing overlaps, so translucent colors are even.
use glam::{Vec2, vec2};

use super::Vector;
use crate::{
    anim_object::render::PipelineKind,
    renderer::{Index, Vertex},
};
use scal_core::Color;

/// Corners sharper than this get their miter cut off, so they do not spike out.
const MITER_LIMIT: f32 = 2.5;
/// More dashes than this means the pattern is too fine to see, draw a solid line instead.
const MAX_DASHES: f32 = 4000.0;
/// Points closer together than this count as one point.
const EPSILON: f32 = 1e-4;

fn push_quad_vertex(vertices: &mut Vec<Vertex>, position: Vec2, color: Color) {
    // uv.y > 0.5 selects the plain fill path of the shape shader.
    vertices.push(Vertex {
        position,
        color,
        uv: vec2(0.0, 1.0),
    });
}

fn dedupe(points: &[Vec2]) -> Vec<Vec2> {
    let mut out: Vec<Vec2> = Vec::with_capacity(points.len());
    for &p in points {
        if out.last().is_none_or(|last| last.distance(p) > EPSILON) {
            out.push(p);
        }
    }
    out
}

fn normal(direction: Vec2) -> Vec2 {
    vec2(-direction.y, direction.x)
}

/// Strokes the polyline as one strip of quads.
fn stroke(
    points: &[Vec2],
    half: f32,
    color: Color,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<Index>,
) {
    let points = dedupe(points);
    if points.len() < 2 {
        return;
    }
    let directions: Vec<Vec2> = points
        .windows(2)
        .map(|w| (w[1] - w[0]).normalize())
        .collect();

    let base = vertices.len() as u32;
    for (i, &p) in points.iter().enumerate() {
        let offset = if i == 0 {
            normal(directions[0]) * half
        } else if i == points.len() - 1 {
            normal(directions[i - 1]) * half
        } else {
            let n0 = normal(directions[i - 1]);
            let n1 = normal(directions[i]);
            let sum = n0 + n1;
            if sum.length() < EPSILON {
                // The line turns around on itself.
                n0 * half
            } else {
                let miter = sum.normalize();
                let scale = (1.0 / miter.dot(n0)).min(MITER_LIMIT);
                miter * half * scale
            }
        };
        push_quad_vertex(vertices, p + offset, color);
        push_quad_vertex(vertices, p - offset, color);
    }
    for i in 0..points.len() as u32 - 1 {
        let a = base + i * 2;
        indices.extend([a, a + 1, a + 3, a, a + 3, a + 2]);
    }
}

/// The part of the polyline between the distances `from` and `to` along it.
fn slice(points: &[Vec2], from: f32, to: f32) -> Vec<Vec2> {
    let mut out = Vec::new();
    let mut walked = 0.0;
    for w in points.windows(2) {
        let len = w[0].distance(w[1]);
        let (start, end) = (walked, walked + len);
        walked = end;
        if end <= from || start >= to || len <= 0.0 {
            continue;
        }
        let at = |d: f32| w[0].lerp(w[1], ((d - start) / len).clamp(0.0, 1.0));
        if out.is_empty() {
            out.push(at(from));
        }
        out.push(at(to));
    }
    out
}

fn length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}

pub fn generate_vector_mesh(v: &Vector) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut points = dedupe(&v.points);
    let half = (v.thickness * 0.5).max(0.0);

    // The arrow head sits on the last segment and points along it, from the second to last point
    // to the last one. The line stops where the head starts.
    if v.arrow_head && points.len() >= 2 && v.head_length > 0.0 {
        let tip = points[points.len() - 1];
        let before = points[points.len() - 2];
        let segment = tip - before;
        let segment_len = segment.length();
        let direction = segment / segment_len;
        // A last segment shorter than the head shrinks the whole head, so it never gets longer
        // than the segment it sits on.
        let fit = (segment_len / v.head_length).min(1.0);
        let head_len = v.head_length * fit;
        let head_half_width = v.head_width * fit * 0.5;
        let base_center = tip - direction * head_len;
        let side = normal(direction) * head_half_width;

        let base = vertices.len() as u32;
        push_quad_vertex(&mut vertices, tip, v.color);
        push_quad_vertex(&mut vertices, base_center + side, v.color);
        push_quad_vertex(&mut vertices, base_center - side, v.color);
        indices.extend([base, base + 1, base + 2]);

        let last = points.len() - 1;
        points[last] = base_center;
        points = dedupe(&points);
    }

    let total = length(&points);
    match v.dash {
        Some((dash, gap))
            if dash > 0.0 && gap >= 0.0 && total / (dash + gap).max(EPSILON) < MAX_DASHES =>
        {
            let mut start = 0.0;
            while start < total {
                let end = (start + dash).min(total);
                stroke(
                    &slice(&points, start, end),
                    half,
                    v.color,
                    &mut vertices,
                    &mut indices,
                );
                start += dash + gap;
            }
        }
        _ => stroke(&points, half, v.color, &mut vertices, &mut indices),
    }

    (vertices, indices, PipelineKind::Shape)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim_object::Transform;
    use glam::Vec3;

    fn vector(points: &[Vec2]) -> Vector {
        Vector {
            points: points.to_vec(),
            color: Color::WHITE,
            thickness: 10.0,
            arrow_head: false,
            head_length: 40.0,
            head_width: 30.0,
            dash: None,
            transform: Transform::with_uuid(uuid::Uuid::new_v4(), Vec3::ZERO),
        }
    }

    fn bounds(vertices: &[Vertex]) -> (Vec2, Vec2) {
        vertices.iter().fold(
            (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
            |(min, max), v| (min.min(v.position), max.max(v.position)),
        )
    }

    #[test]
    fn straight_line_is_one_quad_with_the_thickness() {
        let v = vector(&[vec2(0.0, 0.0), vec2(100.0, 0.0)]);
        let (vertices, indices, _) = generate_vector_mesh(&v);
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        let (min, max) = bounds(&vertices);
        assert_eq!((min, max), (vec2(0.0, -5.0), vec2(100.0, 5.0)));
    }

    #[test]
    fn arrow_head_points_from_the_second_to_last_point_to_the_last() {
        let mut v = vector(&[vec2(0.0, 0.0), vec2(100.0, 0.0), vec2(100.0, 100.0)]);
        v.arrow_head = true;
        let (vertices, _, _) = generate_vector_mesh(&v);
        // The tip is the first vertex: the end of the line, reached going down.
        assert_eq!(vertices[0].position, vec2(100.0, 100.0));
        // Head base is 40 back along the last segment, 30 wide.
        let corners = [vertices[1].position, vertices[2].position];
        assert!(corners.contains(&vec2(115.0, 60.0)) && corners.contains(&vec2(85.0, 60.0)));
        // The line ends at the base of the head, not under its tip.
        let (_, max) = bounds(&vertices[3..]);
        assert!((max.y - 60.0).abs() < 1e-3, "line reaches y={}", max.y);
    }

    #[test]
    fn short_last_segment_shrinks_the_head() {
        let mut v = vector(&[vec2(0.0, 0.0), vec2(100.0, 0.0), vec2(120.0, 0.0)]);
        v.arrow_head = true;
        let (vertices, _, _) = generate_vector_mesh(&v);
        // 20 long instead of 40, so half as wide too.
        assert_eq!(vertices[0].position, vec2(120.0, 0.0));
        let half_width = (vertices[1].position.y - vertices[2].position.y).abs() / 2.0;
        assert!((half_width - 7.5).abs() < 1e-3);
        assert!((vertices[1].position.x - 100.0).abs() < 1e-3);
    }

    #[test]
    fn dashes_are_separate_pieces_that_stay_on_the_line() {
        let mut v = vector(&[vec2(0.0, 0.0), vec2(100.0, 0.0)]);
        v.dash = Some((10.0, 10.0));
        let (vertices, _, _) = generate_vector_mesh(&v);
        // 0-10, 20-30, ... 80-90: five dashes, four vertices each.
        assert_eq!(vertices.len(), 5 * 4);
        let (min, max) = bounds(&vertices);
        assert_eq!((min.x, max.x), (0.0, 90.0));
    }

    #[test]
    fn corners_do_not_overlap_or_spike_and_degenerate_input_is_safe() {
        let v = vector(&[vec2(0.0, 0.0), vec2(100.0, 0.0), vec2(100.0, 100.0)]);
        let (vertices, indices, _) = generate_vector_mesh(&v);
        // One strip: 3 points, 2 vertices each.
        assert_eq!(vertices.len(), 6);
        assert_eq!(indices.len(), 12);
        // A mitered right angle reaches 5 past the corner on both axes.
        let (min, max) = bounds(&vertices);
        assert_eq!((min.y, max.x), (-5.0, 105.0));

        // Turning back on itself, repeated points and a single point.
        let back = vector(&[vec2(0.0, 0.0), vec2(50.0, 0.0), vec2(0.0, 0.0)]);
        let (vertices, _, _) = generate_vector_mesh(&back);
        assert!(vertices.iter().all(|v| v.position.is_finite()));
        let same = vector(&[vec2(5.0, 5.0), vec2(5.0, 5.0)]);
        assert!(generate_vector_mesh(&same).0.is_empty());
        let mut arrow = vector(&[vec2(5.0, 5.0), vec2(5.0, 5.0)]);
        arrow.arrow_head = true;
        assert!(generate_vector_mesh(&arrow).0.is_empty());
    }
}
