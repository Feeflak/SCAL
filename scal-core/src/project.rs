use serde::{Deserialize, Serialize};

use crate::anim_op::AnimOP;
use crate::camera::Camera;
use crate::color::Color;
use crate::theme::Theme;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Declares the whole animation.
/// Gets transported over IPC to the runtime to get previewed / rendered
///
#[allow(missing_docs)]
pub struct Project {
    pub scene_settings: SceneSettings,
    pub timeline: Vec<AnimOP>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct SceneSettings {
    pub background_color: Color,
    pub camera: Camera,
    pub default_theme: Theme,
}
