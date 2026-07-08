use serde::{Deserialize, Serialize};

use crate::anim_op::AnimOP;
use crate::camera::Camera;
use crate::color::Color;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub scene_settings: SceneSettings,
    pub timeline: Vec<AnimOP>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneSettings {
    pub background_color: Color,
    pub camera: Camera,
}
