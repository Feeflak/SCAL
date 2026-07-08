use serde::{Deserialize, Serialize};

use crate::seconds::Seconds;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sfx {
    pub path: String,
    pub volume: f32,
    pub pitch: f32,
    pub time_offset: Seconds,
    pub duration: Seconds,
    pub pitch_variation: f32,
}

impl Sfx {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            volume: 1.0,
            pitch: 1.0,
            time_offset: 0.0,
            duration: 0.0,
            pitch_variation: 0.0,
        }
    }
}
