use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_obj::AnimObj;
use crate::color::Color;
use crate::ease::Ease;
use crate::seconds::Seconds;
use crate::sfx::Sfx;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceLoc {
    pub file: String,
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnimOP {
    Instantiate(AnimObj, Option<SourceLoc>),
    TransformMovePos(Uuid, Vec2, Seconds, Ease, Option<SourceLoc>),
    TransformMoveToObj(Uuid, Uuid, Vec2, Seconds, Ease, Option<SourceLoc>),
    TransformRotate(Uuid, f32, Seconds, Ease, Option<SourceLoc>),
    TransformScale(Uuid, Vec2, Seconds, Ease, Option<SourceLoc>),
    CodeAddLines(
        Uuid,
        String,
        usize,
        Seconds,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeModifyLine(
        Uuid,
        u32,
        String,
        Seconds,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeRemoveLines(
        Uuid,
        Range<u32>,
        Seconds,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeHighlight(Uuid, CodeHighlightAction, Option<SourceLoc>),
    All(Vec<AnimOP>, Option<SourceLoc>),
    Sequence(Vec<AnimOP>, Option<SourceLoc>),
    Wait(Seconds, Option<SourceLoc>),
    PlaySound(Sfx, Seconds, Option<SourceLoc>),
}

impl AnimOP {
    pub fn location(&self) -> Option<&SourceLoc> {
        match self {
            AnimOP::Instantiate(_, l)
            | AnimOP::TransformMovePos(_, _, _, _, l)
            | AnimOP::TransformMoveToObj(_, _, _, _, _, l)
            | AnimOP::TransformRotate(_, _, _, _, l)
            | AnimOP::TransformScale(_, _, _, _, l)
            | AnimOP::CodeAddLines(_, _, _, _, _, _, l)
            | AnimOP::CodeModifyLine(_, _, _, _, _, _, l)
            | AnimOP::CodeRemoveLines(_, _, _, _, _, l)
            | AnimOP::CodeHighlight(_, _, l)
            | AnimOP::All(_, l)
            | AnimOP::Sequence(_, l)
            | AnimOP::Wait(_, l)
            | AnimOP::PlaySound(_, _, l) => l.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CodeAnimationStyle {
    TypeWriter,
    TypeWriterInstantResize,
    Fold,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CodeHighlightAction {
    Lines {
        ranges: Vec<Range<usize>>,
        color: Color,
        duration: Seconds,
        curve: Ease,
    },
    Pattern {
        regex: String,
        color: Color,
        duration: Seconds,
        curve: Ease,
    },
}

pub trait IntoAnimOp {
    fn into_anim_op(self) -> AnimOP;
}

impl IntoAnimOp for AnimOP {
    fn into_anim_op(self) -> AnimOP {
        self
    }
}

impl IntoAnimOp for PlaySoundBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

impl AnimOP {
    pub fn with_location(mut self, loc: SourceLoc) -> Self {
        match &mut self {
            AnimOP::Instantiate(_, l)
            | AnimOP::TransformMovePos(_, _, _, _, l)
            | AnimOP::TransformMoveToObj(_, _, _, _, _, l)
            | AnimOP::TransformRotate(_, _, _, _, l)
            | AnimOP::TransformScale(_, _, _, _, l)
            | AnimOP::CodeAddLines(_, _, _, _, _, _, l)
            | AnimOP::CodeModifyLine(_, _, _, _, _, _, l)
            | AnimOP::CodeRemoveLines(_, _, _, _, _, l)
            | AnimOP::CodeHighlight(_, _, l)
            | AnimOP::All(_, l)
            | AnimOP::Sequence(_, l)
            | AnimOP::Wait(_, l)
            | AnimOP::PlaySound(_, _, l) => *l = Some(loc),
        }
        self
    }

    pub fn wait(duration: Seconds) -> Self {
        AnimOP::Wait(duration, None)
    }

    pub fn play(sfx: Sfx) -> PlaySoundBuilder {
        PlaySoundBuilder { sfx, delay: 0.0 }
    }
}

pub fn wait(duration: Seconds) -> AnimOP {
    AnimOP::Wait(duration, None)
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
        AnimOP::PlaySound(b.sfx, b.delay, None)
    }
}

#[macro_export]
macro_rules! timeline {
    ( $( $item:expr ),* $(,)? ) => {
        vec![ $( {
            let __op = $crate::IntoAnimOp::into_anim_op($item);
            __op.with_location($crate::SourceLoc {
                file: file!().to_string(),
                line: line!(),
                col: column!(),
            })
        } ),* ]
    };
}

#[macro_export]
macro_rules! parallel {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::All(timeline![ $( $op ),* ], None)
    };
}

#[macro_export]
macro_rules! sequence {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::Sequence(timeline![ $( $op ),* ], None)
    };
}
