use glam::{Mat4, Quat, Vec2, Vec3, vec3};

use crate::anim_object::Transform;

#[derive(Clone, Copy)]
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
        -1.0,
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        0.0,
        1.0,
    ])
}
impl Camera {
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

impl Transform {
    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.pos.x, self.pos.y, self.z),
        )
    }
}
