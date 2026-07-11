use glam::{Mat4, Vec2, vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Camera {
    // Makes the renderer write camera's matrix into buffer
    pub dirty: bool,
    // Size of the space that is used in animations.
    // For example, if the virtual_size= 1080 then objects with transform at y= 540 will be at the
    // center of your screen no mater the render resolution.
    pub virtual_size: Vec2,
    pub position: Vec2,
    pub zoom: f32,
}

impl Camera {
    #[must_use]

    pub fn new(virtual_size: Vec2, position: Vec2, zoom: f32) -> Self {
        Self {
            dirty: true,
            virtual_size,
            position,
            zoom,
        }
    }

    pub fn get_matrix(&self) -> Mat4 {
        let view = Mat4::from_translation(vec3(-self.position.x, -self.position.y, 0.0))
            * Mat4::from_scale(vec3(self.zoom, self.zoom, 1.0));

        let projection = ortho(0.0, self.virtual_size.x, self.virtual_size.y, 0.0);

        projection * view
    }
}

fn ortho(left: f32, right: f32, bottom: f32, top: f32) -> Mat4 {
    Mat4::from_cols_array(&[
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        0.0,
        1.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, vec2, vec3};

    #[test]
    fn camera_center_maps_to_origin() {
        let camera = Camera::new(vec2(1920.0, 1080.0), Vec2::ZERO, 1.0);
        let mat = camera.get_matrix();
        let center = mat * vec3(960.0, 540.0, 0.0).extend(1.0);
        assert!((center.x).abs() < 0.001, "center.x={}", center.x);
        assert!((center.y).abs() < 0.001, "center.y={}", center.y);
    }

    #[test]
    fn camera_top_left_maps_to_neg_one_pos_one() {
        let camera = Camera::new(vec2(1920.0, 1080.0), Vec2::ZERO, 1.0);
        let mat = camera.get_matrix();
        let tl = mat * vec3(0.0, 0.0, 0.0).extend(1.0);
        assert!((tl.x + 1.0).abs() < 0.001, "tl.x={}", tl.x);
        assert!((tl.y - 1.0).abs() < 0.001, "tl.y={}", tl.y);
    }

    #[test]
    fn camera_bottom_right_maps_to_one_neg_one() {
        let camera = Camera::new(vec2(1920.0, 1080.0), Vec2::ZERO, 1.0);
        let mat = camera.get_matrix();
        let br = mat * vec3(1920.0, 1080.0, 0.0).extend(1.0);
        assert!((br.x - 1.0).abs() < 0.001, "br.x={}", br.x);
        assert!((br.y + 1.0).abs() < 0.001, "br.y={}", br.y);
    }

    #[test]
    fn camera_with_position_shifts_view() {
        let camera = Camera::new(vec2(1920.0, 1080.0), vec2(100.0, 200.0), 1.0);
        let mat = camera.get_matrix();
        let origin = mat * vec3(0.0, 0.0, 0.0).extend(1.0);
        let expected_x = 2.0 * (-100.0) / 1920.0 - 1.0;
        let expected_y = 1.0 - 2.0 * (-200.0) / 1080.0;
        assert!(
            (origin.x - expected_x).abs() < 0.001,
            "origin.x={}, expected={}",
            origin.x,
            expected_x
        );
        assert!(
            (origin.y - expected_y).abs() < 0.001,
            "origin.y={}, expected={}",
            origin.y,
            expected_y
        );
    }

    #[test]
    fn camera_zoom_scales_view() {
        let camera = Camera::new(vec2(1920.0, 1080.0), Vec2::ZERO, 2.0);
        let mat = camera.get_matrix();
        let center = mat * vec3(960.0, 540.0, 0.0).extend(1.0);
        let expected_x = 2.0 * (2.0 * 960.0) / 1920.0 - 1.0;
        let expected_y = 1.0 - 2.0 * (2.0 * 540.0) / 1080.0;
        assert!(
            (center.x - expected_x).abs() < 0.001,
            "center.x={}, expected={}",
            center.x,
            expected_x
        );
        assert!(
            (center.y - expected_y).abs() < 0.001,
            "center.y={}, expected={}",
            center.y,
            expected_y
        );
    }

    #[test]
    fn transform_local_matrix_identity() {
        let t = Transform::new(None, Vec3::ZERO, 0.0, Vec2::ONE);
        let mat = t.get_local_matrix();
        let p = mat * vec3(1.0, 2.0, 0.0).extend(1.0);
        assert!((p.x - 1.0).abs() < 0.001);
        assert!((p.y - 2.0).abs() < 0.001);
    }

    #[test]
    fn transform_local_matrix_scale() {
        let t = Transform::new(None, Vec3::ZERO, 0.0, vec2(2.0, 3.0));
        let mat = t.get_local_matrix();
        let p = mat * vec3(1.0, 1.0, 0.0).extend(1.0);
        assert!((p.x - 2.0).abs() < 0.001);
        assert!((p.y - 3.0).abs() < 0.001);
    }

    #[test]
    fn transform_local_matrix_translation() {
        let t = Transform::new(None, vec3(100.0, 200.0, 5.0), 0.0, Vec2::ONE);
        let mat = t.get_local_matrix();
        let p = mat * vec3(0.0, 0.0, 0.0).extend(1.0);
        assert!((p.x - 100.0).abs() < 0.001);
        assert!((p.y - 200.0).abs() < 0.001);
        assert!((p.z - 5.0).abs() < 0.001);
    }

    #[test]
    fn transform_local_matrix_rotation_90() {
        let t = Transform::new(None, Vec3::ZERO, 90.0, Vec2::ONE);
        let mat = t.get_local_matrix();
        let p = mat * vec3(1.0, 0.0, 0.0).extend(1.0);
        assert!((p.x).abs() < 0.001, "p.x={}", p.x);
        assert!((p.y - 1.0).abs() < 0.001, "p.y={}", p.y);
    }

    #[test]
    fn transform_local_matrix_scale_and_translation() {
        let t = Transform::new(None, vec3(50.0, -30.0, 0.0), 0.0, vec2(2.0, 0.5));
        let mat = t.get_local_matrix();
        let p = mat * vec3(10.0, 20.0, 0.0).extend(1.0);
        assert!((p.x - 70.0).abs() < 0.001);
        assert!((p.y - (-20.0)).abs() < 0.001);
    }
}
