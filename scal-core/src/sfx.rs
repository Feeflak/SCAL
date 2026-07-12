use serde::{Deserialize, Serialize};

use crate::anim_builders::PlaySoundBuilder;
use crate::seconds::Time;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A reusable sound-effect that can be played during an animation
/// Sfx don't need to be instantiated.
pub struct Sfx {
    /// File path to the audio file
    pub path: String,
    /// Volume multiplier (1.0 = normal)
    pub volume: f32,
    /// Pitch multiplier (1.0 = normal)
    pub pitch: f32,
    /// Time offset to skip into the audio file
    pub time_offset: Time,
    /// Duration of the audio to play
    pub duration: Time,
    /// Random pitch variation applied each time the sound is played
    pub pitch_variation: f32,
}

/// Create a new sound effect builder.
/// ```
/// sfx()
///     .path("./click.mp3")
///     .volume(0.5)
///     .play()
///     .after(0.3.s()),
/// ```
#[must_use]
pub fn sfx() -> SfxBuilder {
    SfxBuilder::default()
}

/// Builder for constructing a sound effect
pub struct SfxBuilder {
    path: String,
    volume: f32,
    pitch: f32,
    time_offset: Time,
    duration: Time,
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

#[allow(clippy::return_self_not_must_use)]
impl SfxBuilder {
    #[must_use]
    /// Set the filepath of the audio file
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    #[must_use]
    /// Set the volume multiplier
    pub const fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
    #[must_use]
    /// Set the pitch multiplier
    pub const fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }
    #[must_use]
    /// Skip a time offset into the audio file
    pub const fn skip_time(mut self, t: Time) -> Self {
        self.time_offset = t;
        self
    }
    #[must_use]
    /// Set the duration of audio to play (0.0 plays the full file)
    pub const fn duration(mut self, dur: Time) -> Self {
        self.duration = dur;
        self
    }
    #[must_use]
    /// Set a random pitch variation applied each time the sound is played
    pub const fn pitch_variation(mut self, var: f32) -> Self {
        self.pitch_variation = var;
        self
    }
    #[must_use]
    /// Consume the builder and return a `PlaySoundBuilder` to configure playback
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
