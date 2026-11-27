//! Mesh of a [`Vector`]: the line as a strip of quads with mitered corners, cut into dashes if
//! asked, and the arrow head as one triangle or as an open rounded `>`. Nothing overlaps, so
//! translucent colors are even.
use std::f32::consts::PI;

use glam::{Vec2, vec2};

use super::Vector;
use crate::{
    anim_object::render::PipelineKind,
    renderer::{Index, Vertex},
};
use scal_core::{Color, HeadStyle};

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

/// Triangle fan around `center` over the arc that starts at `center + start` and turns by `sweep`
/// radians (counter clockwise when positive).
fn push_fan(
    center: Vec2,
    start: Vec2,
    sweep: f32,
    color: Color,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<Index>,
) {
    // One step per 15 degrees
    let steps = ((sweep.abs() / (PI / 12.0)).ceil() as u32).max(1);
    let base = vertices.len() as u32;
    push_quad_vertex(vertices, center, color);
    for i in 0..=steps {
        let angle = sweep * i as f32 / steps as f32;
        push_quad_vertex(vertices, center + Vec2::from_angle(angle).rotate(start), color);
    }
    for i in 0..steps {
        indices.extend([base, base + 1 + i, base + 2 + i]);
    }
}

fn push_triangle(
    corners: [Vec2; 3],
    color: Color,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<Index>,
) {
    let base = vertices.len() as u32;
    for corner in corners {
        push_quad_vertex(vertices, corner, color);
    }
    indices.extend([base, base + 1, base + 2]);
}

/// Pushes an open `>` pointing along `direction`: two strokes of width `2 * half` meeting at a
/// rounded corner, with rounded ends. Its tip, the furthest point of the round corner, is at
/// `tip`. It is `length` long and `width` wide in total. Returns how far behind `tip` the
/// line that leads to it has to stop, the line should end with `push_head_joint`.
///
/// The parts only touch, they never overlap.
fn push_rounded_head(
    tip: Vec2,
    direction: Vec2,
    length: f32,
    width: f32,
    half: f32,
    color: Color,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<Index>,
) -> RoundedHead {
    let side = normal(direction);
    // The corner of the strokes' center lines, the round corner reaches `half` past it.
    let apex = tip - direction * half;
    let axial = (length - half).max(0.0);
    let lateral = (width * 0.5 - half).max(0.0);

    if lateral < EPSILON || axial < EPSILON {
        // Too narrow to open up, just a round end of the line.
        push_fan(apex, -side * half, PI, color, vertices, indices);
        return RoundedHead {
            line_end_back: half,
            joint: None,
        };
    }

    let a = apex - direction * axial + side * lateral;
    let b = apex - direction * axial - side * lateral;
    let d0 = (apex - a).normalize();
    let d1 = (b - apex).normalize();
    let (n0, n1) = (normal(d0), normal(d1));
    // +1 when the head turns left at the apex, -1 when it turns right. The inner side of the
    // corner is on the side it turns to.
    let turn = d0.perp_dot(d1).signum();
    let miter = (n0 + n1).normalize();
    let inner_offset = turn * miter * half / miter.dot(n0);
    let inner = apex + inner_offset;
    let outer0 = apex - turn * n0 * half;
    let outer1 = apex - turn * n1 * half;
    let sweep = d0.dot(d1).clamp(-1.0, 1.0).acos() * turn;

    // Both strokes, up to the inner and outer corner.
    for (from, to_outer, nrm) in [(a, outer0, n0), (b, outer1, n1)] {
        let near = from + turn * nrm * half;
        let far = from - turn * nrm * half;
        let base = vertices.len() as u32;
        for p in [near, far, to_outer, inner] {
            push_quad_vertex(vertices, p, color);
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    // Round the outer corner and fill the two triangles between it and the inner corner.
    push_fan(apex, outer0 - apex, sweep, color, vertices, indices);
    push_triangle([apex, inner, outer0], color, vertices, indices);
    push_triangle([apex, outer1, inner], color, vertices, indices);
    // Round ends.
    push_fan(a, n0 * half, PI, color, vertices, indices);
    push_fan(b, -n1 * half, PI, color, vertices, indices);

    // Where the line has to stop: where the inner edges of the strokes are as far from the axis as
    // the edges of the line. The corner of the head sticks into the line from there.
    let inner_back = (tip - inner).dot(direction);
    let line_end_back = inner_back + half * axial / lateral;
    RoundedHead {
        line_end_back,
        joint: Some(inner_back),
    }
}

/// What `push_rounded_head` leaves for the line to finish.
struct RoundedHead {
    /// Distance from the tip to where the line has to stop.
    line_end_back: f32,
    /// Distance from the tip to the inner corner, where the end of the line comes to a point.
    joint: Option<f32>,
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
        let end = points[points.len() - 1];
        let before = points[points.len() - 2];
        let segment = end - before;
        let segment_len = segment.length();
        let direction = segment / segment_len;
        // A last segment shorter than the head shrinks the whole head, so it never gets longer
        // than the segment it sits on.
        let fit = (segment_len / v.head_length).min(1.0);
        let head_len = v.head_length * fit;
        let head_width = v.head_width * fit;
        // The head animation slides the head back, but never past the point before it.
        let slide = v
            .head_animation
            .map_or(0.0, |a| {
                a.progress(v.time) * a.distance.unwrap_or(v.head_length * 0.4).max(0.0)
            })
            .min(segment_len - head_len);
        let tip = end - direction * slide;

        let last = points.len() - 1;
        match v.head_style {
            HeadStyle::Filled => {
                let base_center = tip - direction * head_len;
                let side = normal(direction) * (head_width * 0.5);
                push_triangle(
                    [tip, base_center + side, base_center - side],
                    v.color,
                    &mut vertices,
                    &mut indices,
                );
                points[last] = base_center;
            }
            HeadStyle::Rounded => {
                let head = push_rounded_head(
                    tip,
                    direction,
                    head_len,
                    head_width,
                    half,
                    v.color,
                    &mut vertices,
                    &mut indices,
                );
                let room = segment_len - slide;
                points[last] = tip - direction * head.line_end_back.min(room);
                if let Some(joint) = head.joint
                    && head.line_end_back <= room
                {
                    // The line ends with a point that fits between the two strokes of the head.
                    let side = normal(direction) * half;
                    push_triangle(
                        [
                            points[last] + side,
                            tip - direction * joint,
                            points[last] - side,
                        ],
                        v.color,
                        &mut vertices,
                        &mut indices,
                    );
                }
            }
        }
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
            head_style: HeadStyle::Filled,
            head_animation: None,
            time: 0.0,
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

    fn rounded(points: &[Vec2]) -> Vector {
        Vector {
            arrow_head: true,
            head_style: HeadStyle::Rounded,
            ..vector(points)
        }
    }

    /// How many triangles cover `p`.
    fn coverage(vertices: &[Vertex], indices: &[Index], p: Vec2) -> u32 {
        let side = |a: Vec2, b: Vec2| (b - a).perp_dot(p - a);
        indices
            .chunks(3)
            .filter(|t| {
                let [a, b, c] = [0, 1, 2].map(|i| vertices[t[i] as usize].position);
                let (d1, d2, d3) = (side(a, b), side(b, c), side(c, a));
                (d1 > 0.0 && d2 > 0.0 && d3 > 0.0) || (d1 < 0.0 && d2 < 0.0 && d3 < 0.0)
            })
            .count() as u32
    }

    #[test]
    fn rounded_head_does_not_overlap_the_line_or_itself() {
        for (line, head_len, head_width, thickness) in [
            (vec![vec2(0.0, 0.0), vec2(200.0, 0.0)], 40.0, 30.0, 10.0),
            (vec![vec2(0.0, 0.0), vec2(200.0, 100.0)], 60.0, 50.0, 8.0),
            (vec![vec2(0.0, 0.0), vec2(100.0, 0.0), vec2(100.0, 100.0)], 40.0, 40.0, 6.0),
            (vec![vec2(0.0, 0.0), vec2(200.0, 0.0)], 40.0, 12.0, 10.0),
        ] {
            let mut v = rounded(&line);
            v.head_length = head_len;
            v.head_width = head_width;
            v.thickness = thickness;
            let (vertices, indices, _) = generate_vector_mesh(&v);
            let (min, max) = bounds(&vertices);
            let mut covered = 0;
            let mut y = min.y + 0.0137;
            while y < max.y {
                let mut x = min.x + 0.0291;
                while x < max.x {
                    let c = coverage(&vertices, &indices, vec2(x, y));
                    assert!(c <= 1, "{c} triangles cover ({x}, {y}) of {line:?}");
                    covered += c;
                    x += 0.5;
                }
                y += 0.5;
            }
            assert!(covered > 0);
        }
    }

    #[test]
    fn rounded_head_is_open_and_its_tip_is_at_the_end_of_the_vector() {
        let mut v = rounded(&[vec2(0.0, 0.0), vec2(200.0, 0.0)]);
        v.head_length = 40.0;
        v.head_width = 30.0;
        let (vertices, indices, _) = generate_vector_mesh(&v);
        let (min, max) = bounds(&vertices);
        assert!((max.x - 200.0).abs() < 0.01, "tip at {}", max.x);
        assert!((max.y - min.y - 30.0).abs() < 0.01, "head is 30 wide");
        // The inside of the head is empty, the line is not.
        assert_eq!(coverage(&vertices, &indices, vec2(185.0, 0.3)), 0);
        assert_eq!(coverage(&vertices, &indices, vec2(100.0, 0.3)), 1);
        // The strokes of the head are there.
        assert_eq!(coverage(&vertices, &indices, vec2(190.0, 9.0)), 1);
        assert_eq!(coverage(&vertices, &indices, vec2(190.0, -9.0)), 1);
        // No sharp tip: the corner is cut off round.
        assert_eq!(coverage(&vertices, &indices, vec2(199.9, 4.9)), 0);
    }

    #[test]
    fn head_animation_slides_the_head_back_and_the_line_follows() {
        let mut v = vector(&[vec2(0.0, 0.0), vec2(200.0, 0.0)]);
        v.arrow_head = true;
        v.head_animation = Some(
            scal_core::HeadAnimation::new()
                .distance(30.0)
                .period(1.0)
                .ease(scal_core::Ease::Linear),
        );
        for style in [HeadStyle::Filled, HeadStyle::Rounded] {
            v.head_style = style;
            v.time = 0.0;
            let at_rest = bounds(&generate_vector_mesh(&v).0).1.x;
            assert!((at_rest - 200.0).abs() < 0.01);
            v.time = 0.5;
            let back = bounds(&generate_vector_mesh(&v).0).1.x;
            assert!((back - 170.0).abs() < 0.01, "{style:?} went back to {back}");
            v.time = 1.0;
            assert!((bounds(&generate_vector_mesh(&v).0).1.x - 200.0).abs() < 0.01);
        }
    }

    #[test]
    fn head_never_slides_past_the_point_before_it() {
        let mut v = vector(&[vec2(0.0, 0.0), vec2(50.0, 0.0)]);
        v.arrow_head = true;
        v.head_animation = Some(scal_core::HeadAnimation::new().distance(500.0).ease(scal_core::Ease::Linear));
        v.time = v.head_animation.unwrap().period / 2.0;
        let (vertices, _, _) = generate_vector_mesh(&v);
        assert!(bounds(&vertices).0.x >= -0.01);
        assert!((bounds(&vertices).1.x - 40.0).abs() < 0.01, "50 - (50 - head length 40)");
    }

    #[test]
    #[ignore]
    fn dump_rounded_head() {
        let mut v = rounded(&[vec2(0.0, 0.0), vec2(160.0, 60.0)]);
        v.head_length = 60.0;
        v.head_width = 50.0;
        v.thickness = 8.0;
        let (vertices, indices, _) = generate_vector_mesh(&v);
        let (w, h) = (900usize, 450usize);
        let mut ppm = format!("P3\n{w} {h}\n255\n");
        for py in 0..h {
            for px in 0..w {
                let p = vec2(px as f32 / 5.0 - 10.0, py as f32 / 5.0 - 10.0);
                ppm += if coverage(&vertices, &indices, p) > 0 { "255 255 255 " } else { "20 20 20 " };
            }
            ppm += "\n";
        }
        std::fs::write("/tmp/opencode/head.ppm", ppm).unwrap();
    }
}
