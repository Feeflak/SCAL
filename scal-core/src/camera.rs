use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub virtual_size: Vec2,
    pub position: Vec2,
    pub zoom: f32,
}

impl Camera {
    pub fn new(virtual_size: Vec2, position: Vec2, zoom: f32) -> Self {
        Self {
            virtual_size,
            position,
            zoom,
        }
    }
}
