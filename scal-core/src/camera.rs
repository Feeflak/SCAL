use glam::{Mat4, Vec2, vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// Camera LOL
pub struct Camera {
    /// Makes the renderer write camera's matrix into buffer
    pub dirty: bool,
    // Size of the space that is used in animations.
    /// For example, if the `virtual_size`= 1080 then objects with transform at y= 540 will be at the
    /// center of your screen no mater the render resolution.
    pub virtual_size: Vec2,
    /// Offsets the whole scene
    pub position: Vec2,
    /// Zooms in on the whole scene
    pub zoom: f32,
}

impl Camera {
    #[must_use]
    /// ``virtual_size``- Size of the space that is used in animations.
    /// For example, if the `virtual_size`= 1080 then objects with transform at y= 540 will be at the
    /// center of your screen no mater the render resolution.
    /// ``zoom``- Zooms in on the whole scene
    pub const fn new(virtual_size: Vec2, position: Vec2, zoom: f32) -> Self {
        Self {
            dirty: true,
            virtual_size,
            position,
            zoom,
        }
    }
    #[must_use]
    /// Internal function go the projection matrix
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
