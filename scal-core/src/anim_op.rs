use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_obj::AnimObj;
use crate::color::Color;
use crate::ease::Ease;
use crate::sfx::Sfx;
use crate::seconds::Seconds;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnimOP {
    Instantiate(AnimObj),
    TransformMovePos(Uuid, Vec2, Seconds, Ease),
    TransformMoveToObj(Uuid, Uuid, Vec2, Seconds, Ease),
    TransformRotate(Uuid, f32, Seconds, Ease),
    TransformScale(Uuid, Vec2, Seconds, Ease),
    CodeAddLines(Uuid, String, usize, Seconds, Ease, CodeAnimationStyle),
    CodeModifyLine(Uuid, u32, String, Seconds, Ease, CodeAnimationStyle),
    CodeRemoveLines(Uuid, Range<u32>, Seconds, Ease, CodeAnimationStyle),
    CodeHighlight(Uuid, CodeHighlightAction),
    All(Vec<AnimOP>),
    Sequence(Vec<AnimOP>),
    Wait(Seconds),
    PlaySound(Sfx, Seconds),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CodeAnimationStyle {
    TypeWriter,
    TypeWriterInstantResize,
    Fold,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CodeHighlightAction {
    Lines { ranges: Vec<Range<usize>>, color: Color, duration: Seconds, curve: Ease },
    Pattern { regex: String, color: Color, duration: Seconds, curve: Ease },
}

pub trait IntoAnimOp {
    fn into_anim_op(self) -> AnimOP;
}

impl IntoAnimOp for AnimOP {
    fn into_anim_op(self) -> AnimOP { self }
}

impl IntoAnimOp for PlaySoundBuilder {
    fn into_anim_op(self) -> AnimOP { self.into() }
}

impl AnimOP {
    pub fn wait(duration: Seconds) -> Self {
        AnimOP::Wait(duration)
    }

    pub fn play(sfx: Sfx) -> PlaySoundBuilder {
        PlaySoundBuilder { sfx, delay: 0.0 }
    }
}

pub fn wait(duration: Seconds) -> AnimOP {
    AnimOP::Wait(duration)
}

pub struct PlaySoundBuilder {
    pub(crate) sfx: Sfx,
    pub(crate) delay: Seconds,
}

impl PlaySoundBuilder {
    pub fn after(mut self, delay: Seconds) -> AnimOP {
        self.delay = delay;
        self.into()
    }
    pub fn delay(mut self, delay: Seconds) -> AnimOP {
        self.delay = delay;
        self.into()
    }
}

impl From<PlaySoundBuilder> for AnimOP {
    fn from(b: PlaySoundBuilder) -> AnimOP {
        AnimOP::PlaySound(b.sfx, b.delay)
    }
}

#[macro_export]
macro_rules! timeline {
    ( $( $item:expr ),* $(,)? ) => {
        vec![ $( $crate::IntoAnimOp::into_anim_op($item) ),* ]
    };
}

#[macro_export]
macro_rules! parallel {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::All(timeline![ $( $op ),* ])
    };
}

#[macro_export]
macro_rules! sequence {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::Sequence(timeline![ $( $op ),* ])
    };
}
