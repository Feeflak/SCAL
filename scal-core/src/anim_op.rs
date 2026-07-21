#![allow(missing_docs)]
//! Module With internal type for representing animations + some useful stuff for it

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
    Instantiate(Box<AnimObj>, Option<SourceLoc>),
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
    TerminalTypeInput(Uuid, String, Option<String>, String, String, Time, Ease, Option<CodeAnimationStyle>, Option<SourceLoc>),
    //                  uuid, command, display_override, captured_output, captured_prompt, dur, ease, style, loc
    TerminalOutput(Uuid, TerminalOutputAction, Time, Ease, Option<CodeAnimationStyle>, Option<SourceLoc>),
    //               uuid, action, dur, ease, style, loc
    All(Vec<Self>, Option<SourceLoc>),
    Sequence(Vec<Self>, Option<SourceLoc>),
    Wait(Time, Option<SourceLoc>),
    PlaySound(Sfx, Time, Option<SourceLoc>),
}

impl AnimOP {
    #[must_use]
    pub const fn location(&self) -> Option<&SourceLoc> {
        match self {
            Self::Instantiate(_, l)
            | Self::TransformMovePos(_, _, _, _, l)
            | Self::TransformMoveToObj(_, _, _, _, _, l)
            | Self::TransformRotate(_, _, _, _, l)
            | Self::TransformScale(_, _, _, _, l)
            | Self::CodeAddLines(_, _, _, _, _, _, l)
            | Self::CodeModifyLine(_, _, _, _, _, _, l)
            | Self::CodeRemoveLines(_, _, _, _, _, l)
            | Self::CodeHighlight(_, _, l)
            | Self::TerminalTypeInput(_, _, _, _, _, _, _, _, l)
            | Self::TerminalOutput(_, _, _, _, _, l)
            | Self::All(_, l)
            | Self::Sequence(_, l)
            | Self::Wait(_, l)
            | Self::PlaySound(_, _, l) => l.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Actions for animating terminal output reveal.
pub enum TerminalOutputAction {
    /// Permanently skip N bytes from the start of the output.
    Skip(usize),
    /// Reveal N bytes from the current position (animated).
    Pull(usize),
    /// Append text to the output.
    Push(String),
    /// Reveal all remaining output (animated).
    PullAll,
    /// Reveal output from the current position to the end of the current line (animated).
    PullLine,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeAnimationStyle {
    TypeWriter,
    TypeWriterInstantResize,
    Fold,
    Reveal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CodeHighlightAction {
    Lines {
        ranges: Vec<Range<usize>>,
        color: Color,
        duration: Time,
        curve: Ease,
        clear: bool,
    },
    Pattern {
        regex: String,
        color: Color,
        duration: Time,
        curve: Ease,
        clear: bool,
    },
}
impl CodeHighlightAction {
    #[must_use]
    pub const fn duration_and_curve(&self) -> (Time, Ease) {
        match self {
            Self::Lines {
                ranges: _,
                color: _,
                duration,
                curve,
                ..
            }
            | Self::Pattern {
                regex: _,
                color: _,
                duration,
                curve,
                ..
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

impl AnimOP {
    #[must_use]
    pub fn with_location(mut self, loc: SourceLoc) -> Self {
        match &mut self {
            Self::Instantiate(_, l)
            | Self::TransformMovePos(_, _, _, _, l)
            | Self::TransformMoveToObj(_, _, _, _, _, l)
            | Self::TransformRotate(_, _, _, _, l)
            | Self::TransformScale(_, _, _, _, l)
            | Self::CodeAddLines(_, _, _, _, _, _, l)
            | Self::CodeModifyLine(_, _, _, _, _, _, l)
            | Self::CodeRemoveLines(_, _, _, _, _, l)
            | Self::CodeHighlight(_, _, l)
            | Self::TerminalTypeInput(_, _, _, _, _, _, _, _, l)
            | Self::TerminalOutput(_, _, _, _, _, l)
            | Self::All(_, l)
            | Self::Sequence(_, l)
            | Self::Wait(_, l)
            | Self::PlaySound(_, _, l) => *l = Some(loc),
        }
        self
    }
}
