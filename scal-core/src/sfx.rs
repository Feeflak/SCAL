use serde::{Deserialize, Serialize};

use crate::anim_op::{AnimOP, IntoAnimOp, PlaySoundBuilder};
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

pub fn sfx() -> SfxBuilder {
    SfxBuilder::default()
}

pub struct SfxBuilder {
    path: String,
    volume: f32,
    pitch: f32,
    time_offset: Seconds,
    duration: Seconds,
    pitch_variation: f32,
}

impl Default for SfxBuilder {
    fn default() -> Self {
        Self {
            path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            time_offset: 0.0,
            duration: 0.0,
            pitch_variation: 0.0,
        }
    }
}

impl SfxBuilder {
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }
    pub fn skip_time(mut self, t: Seconds) -> Self {
        self.time_offset = t;
        self
    }
    pub fn duration(mut self, dur: Seconds) -> Self {
        self.duration = dur;
        self
    }
    pub fn pitch_variation(mut self, var: f32) -> Self {
        self.pitch_variation = var;
        self
    }
    pub fn play(self) -> PlaySoundBuilder {
        PlaySoundBuilder {
            sfx: Sfx {
                path: self.path,
                volume: self.volume,
                pitch: self.pitch,
                time_offset: self.time_offset,
                duration: self.duration,
                pitch_variation: self.pitch_variation,
            },
            delay: 0.0,
        }
    }
}
