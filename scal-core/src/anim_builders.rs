use std::collections::HashMap;
use std::ops::Range;
use std::sync::{LazyLock, Mutex};

use uuid::Uuid;

use crate::{
    AnimOP, CodeAnimationStyle, CodeHighlightAction, Color, Ease, IntoAnimOp, ScrollOffsetTarget,
    Sfx, TerminalOutputAction, Time,
};

/// Builder for an animation of adding code lines to the code block.
/// ```ignore
///code.add_lines()
///    .str(
///        r"fn fib(n: u32) -> u32 {
///             match n {
///                 0 => 0,
///                 1 => 1,
///                 _ => fib(n - 1) + fib(n - 2),
///             }
///         }"
///    )
///    .over(5.s())
///    .style(CodeAnimationStyle::TypeWriter),
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
/// ```ignore
///code.modify_line()
///    .str("New Line Contents")
///    .line(25)
///    .over(5.s())
///    .style(CodeAnimationStyle::TypeWriter),
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
/// ```ignore
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
/// ```ignore
///                code.highlight()
///                    .lines(3..6)
///                    .color(Color::new(1.0, 1.0, 0.0, 0.3))
///                    .over(1.s())
///                    .ease(Ease::InOutCubic),
/// ```
///
/// Or with a regex pattern:
/// ```ignore
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
    pub(crate) clear: bool,
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
    #[must_use]
    /// Reset any previous highlights before applying this one
    pub fn reset(mut self) -> Self {
        self.clear = true;
        self.ranges.clear();
        self.regex = None;
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
                clear: b.clear,
            }
        } else {
            CodeHighlightAction::Lines {
                ranges: b.ranges,
                color: b.color,
                duration: b.duration,
                curve: b.ease,
                clear: b.clear,
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

/// Builder for an animation that changes the scroll offset of a scroll layout.
/// ```ignore
/// scrol.scroll().percent(0.3).over(0.1.s()).ease(Ease::OutBack),
/// ```
pub struct ScrollBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<ScrollOffsetTarget>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

impl ScrollBuilder {
    /// Scroll to a percentage of the scrollable distance (0.0 = content start
    /// aligned to the viewport start edge, 1.0 = fully scrolled).
    #[must_use]
    pub const fn percent(mut self, target: f32) -> Self {
        self.target = Some(ScrollOffsetTarget::Percent(target));
        self
    }
    /// Scroll to an absolute pixel offset relative to the viewport start edge.
    #[must_use]
    pub const fn px(mut self, target: f32) -> Self {
        self.target = Some(ScrollOffsetTarget::Pixels(target));
        self
    }
    /// Duration of the scroll animation
    #[must_use]
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    /// Ease function for the scroll animation
    #[must_use]
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<ScrollBuilder> for AnimOP {
    fn from(b: ScrollBuilder) -> Self {
        let target = b.target.unwrap_or(ScrollOffsetTarget::Percent(0.0));
        Self::ScrollOffset(b.uuid, target, b.duration, b.ease, None)
    }
}

impl IntoAnimOp for ScrollBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation of playing a sound effect.
///
/// Sounds are non-blocking: they start at the current timeline cursor (plus any
/// `after`/`delay`) and the timeline continues immediately. Put them in a
/// `parallel![]` block to play them alongside other animations.
///
/// ```ignore
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

/// Builder for an animation of typing a command into a terminal.
/// ```ignore
///                term.input()
///                    .value("ls -la")
///                    .input_view_override("ls")
///                    .over(2.s()),
/// ```
pub struct TerminalInputBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) shell: String,
    pub(crate) source_dir: Option<String>,
    pub(crate) command: String,
    pub(crate) display_override: Option<String>,
    pub(crate) captured_output: String,
    pub(crate) captured_prompt: String,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
    pub(crate) startup_config: Option<String>,
    pub(crate) style: Option<CodeAnimationStyle>,
}

impl TerminalInputBuilder {
    /// Set the command text. Executes the command and captures its output.
    #[must_use]
    pub fn value(mut self, cmd: impl Into<String>) -> Self {
        let cmd: String = cmd.into();
        let session_cwd = SESSIONS.lock().unwrap()
            .get(&self.uuid.to_string())
            .and_then(|s| {
                if s.cwd.as_os_str().is_empty() {
                    None
                } else {
                    Some(s.base_dir.path().join(&s.cwd))
                }
            });
        let prompt = capture_prompt(&self.shell, &self.source_dir, session_cwd.as_deref());
        let output = execute_command(
            &cmd,
            &self.uuid.to_string(),
            &self.shell,
            &self.source_dir,
            &self.startup_config,
        );
        self.command = cmd;
        self.captured_output = output;
        self.captured_prompt = prompt;
        self
    }
    /// Override the displayed command text (visual only, doesn't affect captured output)
    #[must_use]
    pub fn input_view_override(mut self, text: impl Into<String>) -> Self {
        self.display_override = Some(text.into());
        self
    }
    /// Duration of the typing animation
    #[must_use]
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    /// Ease function for the typing animation
    #[must_use]
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    /// Animation style for how the prompt and output appear
    #[must_use]
    pub const fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = Some(style);
        self
    }
}

impl From<TerminalInputBuilder> for AnimOP {
    fn from(b: TerminalInputBuilder) -> Self {
        Self::TerminalTypeInput(
            b.uuid,
            b.command,
            b.display_override,
            b.captured_output,
            b.captured_prompt,
            b.duration,
            b.ease,
            b.style,
            None,
        )
    }
}

impl IntoAnimOp for TerminalInputBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation of revealing terminal output.
/// ```ignore
///                term.output()
///                    .skip(25)
///                    .pull(50)
///                    .over(1.s()),
/// ```
pub struct TerminalOutputBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) action: Option<TerminalOutputAction>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
    pub(crate) style: Option<CodeAnimationStyle>,
}

impl TerminalOutputBuilder {
    /// Permanently skip N bytes from the captured output
    #[must_use]
    pub fn skip(mut self, bytes: usize) -> Self {
        self.action = Some(TerminalOutputAction::Skip(bytes));
        self
    }
    /// Reveal N bytes from the output stream (animated over the duration)
    #[must_use]
    pub fn pull(mut self, bytes: usize) -> Self {
        self.action = Some(TerminalOutputAction::Pull(bytes));
        self
    }
    /// Append extra text to the output stream
    #[must_use]
    pub fn push(mut self, text: impl Into<String>) -> Self {
        self.action = Some(TerminalOutputAction::Push(text.into()));
        self
    }
    /// Reveal all remaining output (animated over the duration)
    #[must_use]
    pub fn pull_all(mut self) -> Self {
        self.action = Some(TerminalOutputAction::PullAll);
        self
    }
    /// Reveal output from the current position to the end of the current line (animated over the duration)
    #[must_use]
    pub fn pull_line(mut self) -> Self {
        self.action = Some(TerminalOutputAction::PullLine);
        self
    }
    /// Duration of the reveal animation
    #[must_use]
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    /// Ease function for the reveal animation
    #[must_use]
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    /// Animation style for how output appears
    #[must_use]
    pub const fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = Some(style);
        self
    }
}

impl From<TerminalOutputBuilder> for AnimOP {
    fn from(b: TerminalOutputBuilder) -> Self {
        let action = b.action.unwrap_or(TerminalOutputAction::PullAll);
        Self::TerminalOutput(b.uuid, action, b.duration, b.ease, b.style, None)
    }
}

impl IntoAnimOp for TerminalOutputBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

fn capture_prompt(shell: &str, source_dir: &Option<String>, cwd: Option<&std::path::Path>) -> String {
    use std::process::Command;
    let work_dir = cwd.or_else(|| source_dir.as_ref().map(std::path::Path::new));
    let prompt_cmds = ["starship prompt 2>/dev/null", "fish_prompt 2>/dev/null"];
    for prompt_cmd in &prompt_cmds {
        if let Ok(out) = Command::new(shell)
            .arg("-c")
            .arg(prompt_cmd)
            .current_dir(work_dir.unwrap_or(std::path::Path::new(".")))
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .env("CLICOLOR_FORCE", "1")
            .env("FORCE_COLOR", "1")
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout).to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    String::new()
}

struct SessionState {
    base_dir: tempfile::TempDir,
    cwd: std::path::PathBuf,
    startup_done: bool,
}

static SESSIONS: LazyLock<Mutex<HashMap<String, SessionState>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn execute_command(
    cmd: &str,
    session_key: &str,
    shell: &str,
    source_dir: &Option<String>,
    startup_config: &Option<String>,
) -> String {
    use std::process::Command;

    let mut sessions = SESSIONS.lock().unwrap();
    let session = if let Some(s) = sessions.get_mut(session_key) {
        s
    } else {
        let dir = tempfile::tempdir();
        let dir = match dir {
            Ok(d) => d,
            Err(e) => return format!("<error: failed to create temp dir: {e}>"),
        };
        if let Some(ref src) = *source_dir {
            let _ = copy_dir_recursive(std::path::Path::new(src), dir.path());
        }
        sessions.insert(
            session_key.to_string(),
            SessionState {
                base_dir: dir,
                cwd: std::path::PathBuf::new(),
                startup_done: false,
            },
        );
        sessions.get_mut(session_key).unwrap()
    };

    let abs_cwd = if session.cwd.as_os_str().is_empty() {
        session.base_dir.path().to_path_buf()
    } else {
        session.base_dir.path().join(&session.cwd)
    };

    let trimmed = cmd.trim();
    if let Some(rest) = trimmed.strip_prefix("cd ") {
        let target = rest.trim();
        if target.is_empty() || target.starts_with('#') {
            return String::new();
        }
        let new_cwd = if target == "-" {
            session.cwd.clone()
        } else {
            let base = if session.cwd.as_os_str().is_empty() {
                std::path::PathBuf::new()
            } else {
                session.cwd.clone()
            };
            let resolved = base.join(target);
            let mut normalized = std::path::PathBuf::new();
            for component in resolved.components() {
                match component {
                    std::path::Component::Normal(c) => normalized.push(c),
                    std::path::Component::ParentDir => { normalized.pop(); }
                    _ => {}
                }
            }
            normalized
        };
        session.cwd = new_cwd;
        return String::new();
    }

    let cmd_str = if let Some(ref config) = *startup_config {
        if !session.startup_done {
            session.startup_done = true;
            format!("{config}\n{cmd}")
        } else {
            cmd.to_string()
        }
    } else {
        cmd.to_string()
    };

    let output = Command::new(shell)
        .arg("-c")
        .arg(&cmd_str)
        .current_dir(&abs_cwd)
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("CLICOLOR_FORCE", "1")
        .env("CARGO_TERM_COLOR", "always")
        .env("FORCE_COLOR", "1")
        .output();

    match output {
        Ok(out) => {
            let mut result = String::new();
            if !out.stdout.is_empty() {
                result.push_str(&sanitize_output(&String::from_utf8_lossy(&out.stdout)));
            }
            if !out.stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&sanitize_output(&String::from_utf8_lossy(&out.stderr)));
            }
            result
        }
        Err(e) => format!("<error: {e}>"),
    }
}

fn sanitize_output(output: &str) -> String {
    let mut result = String::with_capacity(output.len());
    for line in output.split('\n') {
        if let Some(last) = line.rsplit('\r').next() {
            if !last.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(last);
            }
        }
    }
    result
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
