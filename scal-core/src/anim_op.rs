use std::fmt::Display;
use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_obj::AnimObj;
use crate::color::Color;
use crate::ease::Ease;
use crate::seconds::Time;
use crate::sfx::Sfx;

#[derive(Clone)]
pub struct CurrentClosure(pub std::sync::Arc<dyn Fn(AnimObj) -> AnimOP + Send + Sync>);
impl std::fmt::Debug for CurrentClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentClosure").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceLoc {
    pub file: String,
    pub line: u32,
}
impl Display for SourceLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at {}:{}", self.file, self.line)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnimOP {
    Instantiate(AnimObj, Option<SourceLoc>),
    TransformMovePos(Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    TransformMoveToObj(Uuid, Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    TransformRotate(Uuid, f32, Time, Ease, Option<SourceLoc>),
    TransformScale(Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    CodeAddLines(
        Uuid,
        String,
        usize,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeModifyLine(
        Uuid,
        u32,
        String,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeRemoveLines(
        Uuid,
        Range<u32>,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeHighlight(Uuid, CodeHighlightAction, Option<SourceLoc>),
    All(Vec<AnimOP>, Option<SourceLoc>),
    Sequence(Vec<AnimOP>, Option<SourceLoc>),
    Wait(Time, Option<SourceLoc>),
    PlaySound(Sfx, Time, Option<SourceLoc>),
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
        duration: Time,
        curve: Ease,
    },
    Pattern {
        regex: String,
        color: Color,
        duration: Time,
        curve: Ease,
    },
}
impl CodeHighlightAction {
    pub fn duration_and_curve(&self) -> (Time, Ease) {
        match self {
            CodeHighlightAction::Lines {
                ranges: _,
                color: _,
                duration,
                curve,
            } => (*duration, *curve),
            CodeHighlightAction::Pattern {
                regex: _,
                color: _,
                duration,
                curve,
            } => (*duration, *curve),
        }
    }
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

    pub fn wait(duration: Time) -> Self {
        AnimOP::Wait(duration, None)
    }

    pub fn play(sfx: Sfx) -> PlaySoundBuilder {
        PlaySoundBuilder { sfx, delay: 0.0 }
    }
}

pub fn wait(duration: Time) -> AnimOP {
    AnimOP::Wait(duration, None)
}

pub struct PlaySoundBuilder {
    pub(crate) sfx: Sfx,
    pub(crate) delay: Time,
}

impl PlaySoundBuilder {
    pub fn after(mut self, delay: Time) -> AnimOP {
        self.delay = delay;
        self.into()
    }
    pub fn delay(mut self, delay: Time) -> AnimOP {
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
macro_rules! parallel {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::All($crate::timeline![ $( $op ),* ], None)
    };
}

#[macro_export]
macro_rules! sequence {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::Sequence($crate::timeline![ $( $op ),* ], None)
    };
}
