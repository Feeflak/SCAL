use std::ops::Range;

use uuid::Uuid;

use crate::{
    AnimOP, CodeAnimationStyle, CodeHighlightAction, Color, Ease, IntoAnimOp, Sfx, Time,
};

/// Builder for an animation of adding code lines to the code block.
/// ```
///                code.add_lines()
///                    .str(
///                        r"
///fn fib(n: u32) -> u32 {
///    match n {
///        0 => 0,
///        1 => 1,
///        _ => fib(n - 1) + fib(n - 2),
///    }
///}
///                "
///                    )
///                    .over(5.s())
///                    .style(CodeAnimationStyle::TypeWriter),
/// ```
pub struct CodeAddLinesBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) text: String,
    pub(crate) from_line: usize,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
    pub(crate) style: CodeAnimationStyle,
}

impl CodeAddLinesBuilder {
    #[must_use]
    /// Value of the lines that you want to add
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    #[must_use]
    /// Line number that you want to start adding newlines from
    pub const fn from_line(mut self, line: usize) -> Self {
        self.from_line = line;
        self
    }
    #[must_use]
    /// Duration that you want to be animating adding newlines over
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    /// Ease function that you want to use on this animation
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    /// Style of the adding code animation
    pub const fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeAddLinesBuilder> for AnimOP {
    fn from(b: CodeAddLinesBuilder) -> Self {
        Self::CodeAddLines(
            b.uuid,
            b.text,
            b.from_line,
            b.duration,
            b.ease,
            b.style,
            None,
        )
    }
}

impl IntoAnimOp for CodeAddLinesBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation of modifying a line in the code block.
/// ```
///                code.modify_line()
///                    .str("New Line Contents")
///                    .line(25)
///                    .over(5.s())
///                    .style(CodeAnimationStyle::TypeWriter),
/// ```
pub struct CodeModifyLineBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) line: u32,
    pub(crate) text: String,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
    pub(crate) style: CodeAnimationStyle,
}

impl CodeModifyLineBuilder {
    #[must_use]
    /// New value of the line that you want to modify
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    #[must_use]
    /// Line number that you want to modify
    pub const fn line(mut self, line: u32) -> Self {
        self.line = line;
        self
    }
    #[must_use]
    /// Duration that you want to be animating the modification over
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    /// Ease function that you want to use on this animation
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    /// Style of the code animation
    pub const fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeModifyLineBuilder> for AnimOP {
    fn from(b: CodeModifyLineBuilder) -> Self {
        Self::CodeModifyLine(b.uuid, b.line, b.text, b.duration, b.ease, b.style, None)
    }
}

impl IntoAnimOp for CodeModifyLineBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation of removing lines from a code block.
/// ```
///                code.remove_lines()
///                    .range(0..25)
///                    .over(5.s())
///                    .style(CodeAnimationStyle::TypeWriter),
/// ```
pub struct CodeRemoveLinesBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) range: Range<u32>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
    pub(crate) style: CodeAnimationStyle,
}

impl CodeRemoveLinesBuilder {
    #[must_use]
    /// Range of lines that you want to remove
    pub const fn range(mut self, range: Range<u32>) -> Self {
        self.range = range;
        self
    }
    #[must_use]
    /// Duration that you want to be animating the removal over
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    /// Ease function that you want to use on this animation
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    /// Style of the code animation
    pub const fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeRemoveLinesBuilder> for AnimOP {
    fn from(b: CodeRemoveLinesBuilder) -> Self {
        Self::CodeRemoveLines(b.uuid, b.range, b.duration, b.ease, b.style, None)
    }
}

impl IntoAnimOp for CodeRemoveLinesBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}



/// Builder for an animation of highlighting code by line range or regex pattern.
/// ```
///                code.highlight()
///                    .lines(3..6)
///                    .color(Color::new(1.0, 1.0, 0.0, 0.3))
///                    .over(1.s())
///                    .ease(Ease::InOutCubic),
/// ```
///
/// Or with a regex pattern:
/// ```
///                code.highlight()
///                    .pattern(r"fn \w+\(")
///                    .color(Color::new(0.0, 1.0, 0.0, 0.3))
///                    .over(1.s()),
/// ```
pub struct CodeHighlightBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) ranges: Vec<Range<usize>>,
    pub(crate) regex: Option<String>,
    pub(crate) color: Color,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

impl CodeHighlightBuilder {
    #[must_use]
    /// Highlight a range of lines (can be called multiple times for multiple ranges)
    pub fn lines(mut self, range: Range<usize>) -> Self {
        self.ranges.push(range);
        self
    }
    #[must_use]
    /// Highlight lines matching a regex pattern
    pub fn pattern(mut self, regex: impl Into<String>) -> Self {
        self.regex = Some(regex.into());
        self
    }
    #[must_use]
    /// Color of the highlight overlay
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    #[must_use]
    /// Duration of the highlight animation
    pub fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    /// Ease function for the highlight animation
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<CodeHighlightBuilder> for AnimOP {
    fn from(b: CodeHighlightBuilder) -> Self {
        let action = if let Some(regex) = b.regex {
            CodeHighlightAction::Pattern {
                regex,
                color: b.color,
                duration: b.duration,
                curve: b.ease,
            }
        } else {
            CodeHighlightAction::Lines {
                ranges: b.ranges,
                color: b.color,
                duration: b.duration,
                curve: b.ease,
            }
        };
        Self::CodeHighlight(b.uuid, action, None)
    }
}

impl IntoAnimOp for CodeHighlightBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation of playing a sound effect.
/// ```
///                sfx()
///                    .path("./click.mp3")
///                    .play()
///                    .after(0.5.s()),
/// ```
pub struct PlaySoundBuilder {
    pub(crate) sfx: Sfx,
    pub(crate) delay: Time,
}

impl PlaySoundBuilder {
    #[must_use]
    /// Delay before the sound starts playing, relative to when the animation is triggered
    pub fn after(mut self, delay: Time) -> AnimOP {
        self.delay = delay;
        self.into()
    }
    #[must_use]
    /// Delay before the sound starts playing
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

impl IntoAnimOp for PlaySoundBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}
