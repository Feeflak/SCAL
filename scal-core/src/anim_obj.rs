use std::ops::Range;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_builders::{
    CodeAddLinesBuilder, CodeHighlightBuilder, CodeModifyLineBuilder, CodeMorphBuilder,
    CodeRemoveLinesBuilder, TerminalInputBuilder, TerminalOutputBuilder,
};
use crate::anim_op::{AnimOP, CodeAnimationStyle};
use crate::color::Color;
use crate::ease::Ease;
use crate::motion::{IntoScale, Motion};
use crate::theme::Theme;
use crate::transform::Transform;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Generic for all object type. Conversion is done using ``IntoAnimOp`` trait.
pub struct AnimObj {
    /// Each object has it's UUID so that you can apply animations over IPC to this object's clone in the
    /// runtime.
    pub id: Uuid,
    #[allow(missing_docs)]
    pub transform: Transform,
    #[allow(missing_docs)]
    pub kind: AnimObjKind,
}

impl AnimObj {
    /// Returns an animation that instantly adds that object to the scene.
    /// ```ignore
    /// let pointer = svg()
    ///     .path("./pointer-tool.svg")
    ///     .build();
    /// Project {
    ///    scene_settings,
    ///    timeline: timeline![
    ///        cw.instantiate(),
    ///     ]
    /// }
    /// ```
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(Box::new(self.clone()), None)
    }

    /// Returns an animation that instantly removes this object, and everything attached to it,
    /// from the scene for good. See [`drop!`](crate::drop).
    #[must_use]
    pub fn despawn(&self) -> AnimOP {
        AnimOP::Drop(self.transform.uuid, None)
    }

    /// Shortcut for `transform.position().to(target)`: animate this object to a position.
    /// ```ignore
    /// title.move_to(vec2(0.0, 100.0)).with(SLIDE)
    /// ```
    #[must_use]
    pub fn move_to(&self, target: impl Into<Vec2>) -> crate::transform::PositionBuilder {
        self.transform.position().to(target.into())
    }

    /// Shortcut for `transform.scale().to(..)`: animate this object to a scale. Takes an `f32`
    /// for uniform scale (`0.5` = half size) or a `Vec2`.
    /// ```ignore
    /// title.scale_to(0.5).with(SLIDE)
    /// ```
    #[must_use]
    pub fn scale_to(&self, target: impl IntoScale) -> crate::transform::ScaleBuilder {
        self.transform.scale().to(target.into_scale())
    }

    /// Shortcut for `transform.rotation().to(degrees)`: animate this object to a rotation.
    /// ```ignore
    /// wing.rotate_to(15.0).with(SWEEP)
    /// ```
    #[must_use]
    pub fn rotate_to(&self, degrees: f32) -> crate::transform::RotateBuilder {
        self.transform.rotation().to(degrees)
    }

    /// Scale this object up to full size with [`Motion::POP_IN`]. Pair with `.hidden()` on the
    /// object builder. Override the timing with `.with(..)`, `.over(..)` or `.ease(..)`.
    #[must_use]
    pub fn pop_in(&self) -> crate::transform::ScaleBuilder {
        self.scale_to(1.0).with(Motion::POP_IN)
    }

    /// Scale this object down to nothing with [`Motion::POP_OUT`].
    #[must_use]
    pub fn pop_out(&self) -> crate::transform::ScaleBuilder {
        self.scale_to(0.0).with(Motion::POP_OUT)
    }

    /// Returns a builder for an animation that counts the number shown by this text or math
    /// object. The object has to contain a `{:spec}` placeholder, see [`crate::number_format`].
    /// Works the same for `text()` and `math()`, so one label can be driven by any animation
    /// that shares the same `.over()` and `.ease()`.
    ///
    /// # Panics
    /// If the object is not text or math, or its content has no `{:spec}` placeholder.
    /// ```ignore
    /// let aoa = math(r"\alpha = {:.1}^\circ").build();
    /// parallel![
    ///     wing.transform.rotation().to(-15.0).over(3.s()).ease(Ease::InOutCubic),
    ///     aoa.number().from(0.0).to(15.0).over(3.s()).ease(Ease::InOutCubic),
    /// ]
    /// ```
    #[must_use]
    pub fn number(&self) -> crate::transform::NumberBuilder {
        let has_placeholder = match &self.kind {
            AnimObjKind::Text { value, spans, .. } => {
                crate::number_format::has_placeholder(value)
                    || spans
                        .iter()
                        .any(|s| crate::number_format::has_placeholder(&s.text))
            }
            AnimObjKind::Math { latex, .. } => crate::number_format::has_placeholder(latex),
            _ => panic!("number() only works on text and math objects"),
        };
        assert!(
            has_placeholder,
            "number() needs a placeholder such as {{:.1}} in the object's text or latex"
        );
        crate::transform::NumberBuilder {
            uuid: self.id,
            from: None,
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum AnimObjKind {
    Rectangle {
        size: Vec2,
        corner_radius: f32,
        color: Color,
    },
    Circle {
        radius: f32,
        color: Color,
    },
    Polygon {
        radius: f32,
        sides: u32,
        color: Color,
    },
    Text {
        value: String,
        font_family: String,
        align: Alignment,
        color: Color,
        font_size: f32,
        modifications: Vec<TextModifier>,
        /// Styled sections. When non-empty these are rendered instead of `value`.
        spans: Vec<TextSpan>,
        /// Number shown by the `{:spec}` placeholders in `value`/`spans` (see
        /// [`crate::number_format`]).
        number: f32,
    },
    Math {
        latex: String,
        color: Color,
        font_size: f32,
        display: bool,
        /// Named colors usable as `\textcolor{name}{..}` in `latex`.
        palette: Vec<(String, Color)>,
        /// Number shown by the `{:spec}` placeholders in `latex` (see
        /// [`crate::number_format`]).
        number: f32,
    },
    Code {
        source_code: String,
        font_family: String,
        font_size: f32,
        syntax: Syntax,
        theme: Option<Theme>,
        padding: f32,
        show_line_numbers: bool,
        line_number_color: Color,
    },
    Image {
        path: String,
        size: Vec2,
        color: Color,
        stretch: StretchMode,
    },
    Svg {
        path: String,
        size: Vec2,
        tint: Color,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: Option<f32>,
        stretch: StretchMode,
    },
    CodeWindow {
        source_code: String,
        font_family: String,
        font_size: f32,
        syntax: Syntax,
        theme: Option<Theme>,
        title: String,
        title_font_size: f32,
        width: f32,
        height: f32,
        background_color: Color,
        code_id: Uuid,
        close_btn_id: Uuid,
        minimize_btn_id: Uuid,
        maximize_btn_id: Uuid,
        title_id: Uuid,
        bg_id: Uuid,
        container_id: Uuid,
        title_bar_bg_id: Uuid,
        show_line_numbers: bool,
        line_number_color: Color,
    },
    Terminal {
        shell: String,
        prompt: String,
        font_family: String,
        font_size: f32,
        theme: Option<Theme>,
        width: f32,
        height: f32,
        background_color: Color,
        text_color: Color,
        term_id: Uuid,
        text_buffer_id: Uuid,
        bg_id: Uuid,
        container_id: Uuid,
        close_btn_id: Uuid,
        minimize_btn_id: Uuid,
        maximize_btn_id: Uuid,
        title_id: Uuid,
        title_bar_bg_id: Uuid,
        title: String,
        title_font_size: f32,
        source_dir: Option<String>,
        startup_config: Option<String>,
    },
    Padding {
        size: Vec2,
    },
    Group {
        children: Vec<AnimObj>,
    },
    Layout {
        children: Vec<AnimObj>,
        direction: LayoutDir,
        align: Alignment,
        justify: Alignment,
        gap: f32,
        padding_top: f32,
        padding_bottom: f32,
        padding_left: f32,
        padding_right: f32,
        min_width: f32,
        min_height: f32,
        background_color: Color,
        corner_radius: f32,
    },
    ScrollLayout {
        children: Vec<AnimObj>,
        direction: LayoutDir,
        align: Alignment,
        justify: Alignment,
        gap: f32,
        padding_top: f32,
        padding_bottom: f32,
        padding_left: f32,
        padding_right: f32,
        viewport_width: f32,
        viewport_height: f32,
        show_scrollbar: bool,
        scroll_offset: f32,
        background_color: Color,
        corner_radius: f32,
        mask_children: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash, Eq)]
#[allow(missing_docs)]
pub enum Syntax {
    Rust,
    Nix,
    Python,
    JS,
    Zig,
    Plain,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum StretchMode {
    Fit,
    Fill,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum LayoutDir {
    Column,
    Row,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Alignment {
    Start,
    Center,
    End,
}

/// Wrapper around the ``AnimObj`` struct that allows for clean animation construction for specific
/// ``AnimObj`` kind.
pub struct CodeHandle(pub AnimObj);

impl std::ops::Deref for CodeHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl CodeHandle {
    /// Returns an animation that instantly adds that object to the scene.
    /// ```ignore
    /// let pointer = svg()
    ///     .path("./pointer-tool.svg")
    ///     .build();
    /// Project {
    ///    scene_settings,
    ///    timeline: timeline![
    ///        cw.instantiate(),
    ///     ]
    /// }
    /// ```
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(Box::new(self.0.clone()), None)
    }

    /// Returns a builder for an animation of adding code lines to the code block.
    /// ```ignore
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
    #[must_use]
    pub const fn add_lines(&self) -> CodeAddLinesBuilder {
        CodeAddLinesBuilder {
            uuid: self.0.id,
            text: String::new(),
            from_line: 0,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }

    /// Returns a builder for an animation of modifying a line of code.
    /// ```ignore
    ///                code.modify_line()
    ///                    .str("New Line Contents")
    ///                    .line(25)
    ///                    .over(5.s())
    ///                    .style(CodeAnimationStyle::TypeWriter),
    /// ```
    #[must_use]
    pub const fn modify_line(&self) -> CodeModifyLineBuilder {
        CodeModifyLineBuilder {
            uuid: self.0.id,
            line: 0,
            text: String::new(),
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }

    /// Returns a builder for an animation of highlighting code by line range or regex pattern.
    /// ```ignore
    ///                code.highlight()
    ///                    .lines(3..6)
    ///                    .color(Color::new(1.0, 1.0, 0.0, 0.3))
    ///                    .over(1.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub fn highlight(&self) -> CodeHighlightBuilder {
        CodeHighlightBuilder {
            uuid: self.0.id,
            ranges: Vec::new(),
            regex: None,
            color: Color::new(1.0, 1.0, 0.0, 0.3),
            duration: 1.0,
            ease: Ease::Linear,
            clear: false,
        }
    }

    /// Returns a builder for an animation that morphs the code into other code. Tokens that appear
    /// in both versions glide to their new place and the rest fades. Add `.font_size(..)` to
    /// change the font size in the same motion.
    /// ```ignore
    ///                code.morph_to("let a = b;")
    ///                    .font_size(80.)
    ///                    .over(2.s()),
    /// ```
    #[must_use]
    pub fn morph_to(&self, source: impl Into<String>) -> CodeMorphBuilder {
        CodeMorphBuilder::new(self.0.id).source(source)
    }

    /// Returns a builder for an animation that smoothly changes the font size of the code.
    /// ```ignore
    ///                code.font_size_to(80.).over(1.s()),
    /// ```
    #[must_use]
    pub fn font_size_to(&self, font_size: f32) -> CodeMorphBuilder {
        CodeMorphBuilder::new(self.0.id).font_size(font_size)
    }

    /// Returns a builder for an animation of removing lines form a code block.
    /// ```ignore
    ///                code.remove_lines()
    ///                    .range(0..25)
    ///                    .over(5.s())
    ///                    .style(CodeAnimationStyle::TypeWriter),
    /// ```
    #[must_use]
    pub const fn remove_lines(&self) -> CodeRemoveLinesBuilder {
        CodeRemoveLinesBuilder {
            uuid: self.0.id,
            range: 0..0,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }
}

/// Wrapper around the ``AnimObj`` struct that allows for clean animation construction for specific
/// ``AnimObj`` kind.
pub struct CodeWindowHandle(pub AnimObj);

impl std::ops::Deref for CodeWindowHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl CodeWindowHandle {
    /// Returns an animation that instantly adds that object to the scene.
    /// ```ignore
    /// let pointer = svg()
    ///     .path("./pointer-tool.svg")
    ///     .build();
    /// Project {
    ///    scene_settings,
    ///    timeline: timeline![
    ///        cw.instantiate(),
    ///     ]
    /// }
    /// ```
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(Box::new(self.0.clone()), None)
    }
    /// Returns a builder for an animation of adding code lines to the code block.
    /// ```ignore
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
    #[must_use]
    pub const fn add_lines(&self) -> CodeAddLinesBuilder {
        let code_id = if let AnimObjKind::CodeWindow { code_id, .. } = &self.0.kind {
            *code_id
        } else {
            self.0.id
        };
        CodeAddLinesBuilder {
            uuid: code_id,
            text: String::new(),
            from_line: 0,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }

    /// Returns a builder for an animation of modifying a line of code.
    /// ```ignore
    ///                code.modify_line()
    ///                    .str("New Line Contents")
    ///                    .line(25)
    ///                    .over(5.s())
    ///                    .style(CodeAnimationStyle::TypeWriter),
    /// ```
    #[must_use]
    pub const fn modify_line(&self, line: u32) -> CodeModifyLineBuilder {
        let code_id = if let AnimObjKind::CodeWindow { code_id, .. } = &self.0.kind {
            *code_id
        } else {
            self.0.id
        };
        CodeModifyLineBuilder {
            uuid: code_id,
            line,
            text: String::new(),
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }

    /// Returns a builder for an animation of highlighting code by line range or regex pattern.
    /// ```ignore
    ///                cw.highlight()
    ///                    .lines(3..6)
    ///                    .color(Color::new(1.0, 1.0, 0.0, 0.3))
    ///                    .over(1.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub fn highlight(&self) -> CodeHighlightBuilder {
        let code_id = if let AnimObjKind::CodeWindow { code_id, .. } = &self.0.kind {
            *code_id
        } else {
            self.0.id
        };
        CodeHighlightBuilder {
            uuid: code_id,
            ranges: Vec::new(),
            regex: None,
            color: Color::new(1.0, 1.0, 0.0, 0.3),
            duration: 1.0,
            ease: Ease::Linear,
            clear: false,
        }
    }

    /// Returns a builder for an animation that morphs the code into other code. Tokens that appear
    /// in both versions glide to their new place and the rest fades. Add `.font_size(..)` to
    /// change the font size in the same motion.
    /// ```ignore
    ///                code.morph_to("let a = b;")
    ///                    .font_size(80.)
    ///                    .over(2.s()),
    /// ```
    #[must_use]
    pub fn morph_to(&self, source: impl Into<String>) -> CodeMorphBuilder {
        CodeMorphBuilder::new(self.code_id()).source(source)
    }

    /// Returns a builder for an animation that smoothly changes the font size of the code.
    /// ```ignore
    ///                code.font_size_to(80.).over(1.s()),
    /// ```
    #[must_use]
    pub fn font_size_to(&self, font_size: f32) -> CodeMorphBuilder {
        CodeMorphBuilder::new(self.code_id()).font_size(font_size)
    }

    fn code_id(&self) -> Uuid {
        if let AnimObjKind::CodeWindow { code_id, .. } = &self.0.kind {
            *code_id
        } else {
            self.0.id
        }
    }

    /// Returns a builder for an animation of removing lines form a code block.
    /// ```ignore
    ///                code.remove_lines()
    ///                    .range(0..25)
    ///                    .over(5.s())
    ///                    .style(CodeAnimationStyle::TypeWriter),
    /// ```
    #[must_use]
    pub const fn remove_lines(&self, range: Range<u32>) -> CodeRemoveLinesBuilder {
        let code_id = if let AnimObjKind::CodeWindow { code_id, .. } = &self.0.kind {
            *code_id
        } else {
            self.0.id
        };
        CodeRemoveLinesBuilder {
            uuid: code_id,
            range,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }

    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn close_button(&self) -> CircleHandle {
        if let AnimObjKind::CodeWindow { close_btn_id, .. } = &self.0.kind {
            CircleHandle(*close_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn minimize_button(&self) -> CircleHandle {
        if let AnimObjKind::CodeWindow {
            minimize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*minimize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn maximize_button(&self) -> CircleHandle {
        if let AnimObjKind::CodeWindow {
            maximize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*maximize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn title_text(&self) -> TextHandle {
        if let AnimObjKind::CodeWindow { title_id, .. } = &self.0.kind {
            TextHandle(*title_id)
        } else {
            TextHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn window_background(&self) -> RectangleHandle {
        RectangleHandle(self.0.id)
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn container(&self) -> RectangleHandle {
        if let AnimObjKind::CodeWindow { container_id, .. } = &self.0.kind {
            RectangleHandle(*container_id)
        } else {
            RectangleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to one of the objects that are used inside of the code window. This allows
    /// you to animate them or use the as a reference point for movement.
    pub const fn title_bar_background(&self) -> RectangleHandle {
        if let AnimObjKind::CodeWindow {
            title_bar_bg_id, ..
        } = &self.0.kind
        {
            RectangleHandle(*title_bar_bg_id)
        } else {
            RectangleHandle(self.0.id)
        }
    }
}

/// Wrapper around the ``AnimObj`` struct for a terminal emulator window.
/// The terminal object simulates a terminal emulator window for animating CLI interactions.
/// Commands you write in the animation are actually executed on your machine during
/// animation creation, and their real output is captured and displayed.
///
/// ```ignore
/// let term = terminal()
///     .shell("fish")
///     .prompt("❯ ")
///     .font_family("JetBrains Mono Nerd")
///     .font_size(22.)
///     .width(1500.)
///     .height(800.)
///     .background_color(Color::new(0.08, 0.08, 0.08, 1.0))
///     .source_dir("./fixtures")
///     .pos(WINDOW / 2.)
///     .build();
/// ```
///
/// After building, add it to the timeline:
/// - `term.instantiate()` — adds the terminal window to the scene
/// - `term.input().value("ls -la").over(0.5.s())` — animates typing a command and
///   executes it to capture the real output
/// - `term.input().input_view_override("cargo build --release")` — override the
///   visually displayed text without changing what command gets executed
/// - `term.output().pull_all().over(0.5.s())` — animates revealing all captured output
/// - `term.output().pull(50).over(0.5.s())` — reveals only the first 50 bytes
/// - `term.output().skip(25)` — permanently skips the first 25 bytes of output
/// - `term.output().push("extra text")` — appends custom text to the output
/// - `.source_dir("path")` — copies a directory into a temp working dir before
///   executing commands (useful for reproducing specific environments)
/// - `.startup_config("starship init fish | source")` — sources config before
///   each command (e.g. for prompt customization)
///
/// Full example at `examples/terminal/`.
pub struct TerminalHandle(pub AnimObj);

impl std::ops::Deref for TerminalHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl TerminalHandle {
    /// Returns an animation that instantly adds the terminal to the scene.
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(Box::new(self.0.clone()), None)
    }
    /// Returns a builder for animating typing a command into the terminal.
    /// Use `.value("cmd")` to set the command text (executes it too),
    /// `.input_view_override("text")` to override the visual display,
    /// and `.over(0.2.s())` for the typing duration.
    #[must_use]
    pub fn input(&self) -> TerminalInputBuilder {
        let shell = self.shell();
        let source_dir = self.source_dir();
        let text_buffer_id = if let AnimObjKind::Terminal { text_buffer_id, .. } = &self.0.kind {
            *text_buffer_id
        } else {
            self.0.id
        };
        let startup_config = if let AnimObjKind::Terminal { startup_config, .. } = &self.0.kind {
            startup_config.clone()
        } else {
            None
        };
        TerminalInputBuilder {
            uuid: text_buffer_id,
            shell,
            source_dir,
            command: String::new(),
            display_override: None,
            captured_output: String::new(),
            captured_prompt: String::new(),
            duration: 1.0,
            ease: Ease::Linear,
            startup_config,
            style: None,
        }
    }
    /// Returns a builder for animating terminal output reveal.
    /// Use `.skip(N)`, `.pull(N)`, `.push("text")`, or `.pull_all()`
    /// to control output display, then `.over(0.2.s())` for the duration.
    #[must_use]
    pub fn output(&self) -> TerminalOutputBuilder {
        let text_buffer_id = if let AnimObjKind::Terminal { text_buffer_id, .. } = &self.0.kind {
            *text_buffer_id
        } else {
            self.0.id
        };
        TerminalOutputBuilder {
            uuid: text_buffer_id,
            action: None,
            duration: 1.0,
            ease: Ease::Linear,
            style: None,
        }
    }
    fn shell(&self) -> String {
        if let AnimObjKind::Terminal { shell, .. } = &self.0.kind {
            shell.clone()
        } else {
            "bash".to_string()
        }
    }
    fn source_dir(&self) -> Option<String> {
        if let AnimObjKind::Terminal { source_dir, .. } = &self.0.kind {
            source_dir.clone()
        } else {
            None
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's close button.
    pub const fn close_button(&self) -> CircleHandle {
        if let AnimObjKind::Terminal { close_btn_id, .. } = &self.0.kind {
            CircleHandle(*close_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's minimize button.
    pub const fn minimize_button(&self) -> CircleHandle {
        if let AnimObjKind::Terminal {
            minimize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*minimize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's maximize button.
    pub const fn maximize_button(&self) -> CircleHandle {
        if let AnimObjKind::Terminal {
            maximize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*maximize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's title text.
    pub const fn title_text(&self) -> TextHandle {
        if let AnimObjKind::Terminal { title_id, .. } = &self.0.kind {
            TextHandle(*title_id)
        } else {
            TextHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's title bar background.
    pub const fn title_bar_background(&self) -> RectangleHandle {
        if let AnimObjKind::Terminal {
            title_bar_bg_id, ..
        } = &self.0.kind
        {
            RectangleHandle(*title_bar_bg_id)
        } else {
            RectangleHandle(self.0.id)
        }
    }
    #[must_use]
    /// Returns handle to the terminal window's background rectangle.
    pub const fn window_background(&self) -> RectangleHandle {
        RectangleHandle(self.0.id)
    }
    #[must_use]
    /// Returns handle to the terminal window's container rectangle.
    pub const fn container(&self) -> RectangleHandle {
        if let AnimObjKind::Terminal { container_id, .. } = &self.0.kind {
            RectangleHandle(*container_id)
        } else {
            RectangleHandle(self.0.id)
        }
    }
}

macro_rules! impl_handle {
    ($type:ty) => {
        impl $type {
            /// Returns a builder for an animation that moves this object to a target position.
            /// ```ignore
            ///                handle.position()
            ///                    .to(Vec2::new(100.0, 200.0))
            ///                    .over(5.s())
            ///                    .ease(Ease::InOutCubic),
            /// ```
            #[must_use]
            pub const fn position(&self) -> crate::transform::PositionBuilder {
                crate::transform::PositionBuilder {
                    uuid: self.0,
                    target: None,
                    object: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
            /// Returns a builder for an animation that scales this object.
            /// ```ignore
            ///                handle.scale()
            ///                    .to(Vec2::new(2.0, 2.0))
            ///                    .over(5.s())
            ///                    .ease(Ease::InOutCubic),
            /// ```
            #[must_use]
            pub const fn scale(&self) -> crate::transform::ScaleBuilder {
                crate::transform::ScaleBuilder {
                    uuid: self.0,
                    target: None,
                    object: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
            /// Returns a builder for an animation that rotates this object.
            /// ```ignore
            ///                handle.rotation()
            ///                    .to(360.0)
            ///                    .over(5.s())
            ///                    .ease(Ease::InOutCubic),
            /// ```
            #[must_use]
            pub const fn rotation(&self) -> crate::transform::RotateBuilder {
                crate::transform::RotateBuilder {
                    uuid: self.0,
                    target: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
            /// Shortcut for `.position().to(target)`.
            #[must_use]
            pub fn move_to(&self, target: impl Into<Vec2>) -> crate::transform::PositionBuilder {
                self.position().to(target.into())
            }
            /// Shortcut for `.scale().to(..)`, takes an `f32` or a `Vec2`.
            #[must_use]
            pub fn scale_to(&self, target: impl IntoScale) -> crate::transform::ScaleBuilder {
                self.scale().to(target.into_scale())
            }
            /// Shortcut for `.rotation().to(degrees)`.
            #[must_use]
            pub fn rotate_to(&self, degrees: f32) -> crate::transform::RotateBuilder {
                self.rotation().to(degrees)
            }
            /// Returns a builder for an animation that changes this object's color.
            /// ```ignore
            ///                handle.color()
            ///                    .to(Color::RED)
            ///                    .over(1.s())
            ///                    .ease(Ease::InOutCubic),
            /// ```
            #[must_use]
            pub const fn color(&self) -> crate::transform::ColorBuilder {
                crate::transform::ColorBuilder {
                    uuid: self.0,
                    target: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
        }
        impl From<$type> for Uuid {
            fn from(h: $type) -> Uuid {
                h.0
            }
        }
    };
}

/// Handle to a Circle sub-object of a code window, for use in animation builders.
pub struct CircleHandle(pub Uuid);
/// Handle to a Text sub-object of a code window, for use in animation builders.
pub struct TextHandle(pub Uuid);
/// Handle to a Rectangle sub-object of a code window, for use in animation builders.
pub struct RectangleHandle(pub Uuid);

impl_handle!(CircleHandle);
impl_handle!(TextHandle);
impl_handle!(RectangleHandle);

/// Wrapper around the ``AnimObj`` struct that allows for clean animation construction
/// for scrollable layouts.
pub struct ScrollLayoutHandle(pub AnimObj);

impl std::ops::Deref for ScrollLayoutHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl ScrollLayoutHandle {
    /// Returns an animation that instantly adds the scroll layout to the scene.
    #[must_use]
    pub fn instantiate(&self) -> crate::anim_op::AnimOP {
        crate::anim_op::AnimOP::Instantiate(Box::new(self.0.clone()), None)
    }

    /// Returns a builder for scroll-offset animations.
    /// ```ignore
    /// scrol.scroll().percent(0.3).over(0.1.s()).ease(Ease::OutBack),
    /// scrol.scroll().px(255.).over(0.5.s()).ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub const fn scroll(&self) -> crate::anim_builders::ScrollBuilder {
        crate::anim_builders::ScrollBuilder {
            uuid: self.0.id,
            target: None,
            duration: 1.0,
            ease: crate::ease::Ease::Linear,
        }
    }
}

/// A modification applied to text at creation time (shadow, outline, glow, etc.).
/// Multiple modifications can be added to a single text object.
///
/// Positive `thickness` expands the shape outward (like an outline or drop shadow).
/// Negative `thickness` contracts it inward (like an inset shadow).
/// `softness` controls how blurry/feathered the edges appear by layering copies
/// at decreasing opacity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextModifier {
    /// The color of the modifier effect
    pub color: Color,
    /// How many units the shape is expanded (positive) or contracted (negative)
    pub thickness: f32,
    /// Softness/blur amount — higher values produce softer, more diffuse edges
    pub softness: f32,
    /// Position offset relative to the text origin
    pub pos_offset: Vec3,
    /// Rotation in degrees
    pub rotation: f32,
    /// Scale factor
    pub scale: Vec2,
}

/// A styled section of a rich text object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextSpan {
    /// The text of this section.
    pub text: String,
    /// Color of this section. `None` uses the text object's color.
    pub color: Option<Color>,
    /// Whether this section is drawn bold.
    pub bold: bool,
}

/// Wrapper around the ``AnimObj`` struct for math equations. Derefs to ``AnimObj``, so
/// ``instantiate()`` and ``transform`` work like on every other object.
pub struct MathHandle(
    pub AnimObj,
    /// Default color and dim of [`MathHandle::highlight`], set with `math_style().highlight(..)`.
    pub(crate) (Color, f32),
);

/// Color and dim [`MathHandle::highlight`] uses unless the style or the builder says otherwise.
pub(crate) const DEFAULT_HIGHLIGHT: (Color, f32) = (Color::new(1.0, 0.9, 0.25, 1.0), 0.25);

impl std::ops::Deref for MathHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl MathHandle {
    /// Returns a builder for an animation that changes the color of the equation's
    /// uncolored symbols (colored parts keep their colors; alpha fades everything).
    /// ```ignore
    ///                eq.color()
    ///                    .to(Color::RED)
    ///                    .over(1.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub const fn color(&self) -> crate::transform::ColorBuilder {
        crate::transform::ColorBuilder {
            uuid: self.0.id,
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    /// Returns a handle to a named part of the equation, defined with
    /// ``MathBuilder::var_color``. Every `\textcolor{name}{..}` occurrence is animated together.
    ///
    /// # Panics
    /// If `name` was not defined with `var_color`.
    /// ```ignore
    ///                eq.part("a").color().to(Color::RED).over(1.s()),
    /// ```
    #[must_use]
    pub fn part(&self, name: &str) -> MathPartHandle {
        self.check_parts([name]);
        MathPartHandle {
            uuid: self.0.id,
            name: name.to_string(),
        }
    }

    /// Returns a builder for an animation that highlights the given parts and dims everything
    /// else. The highlight stays until another `highlight` replaces it (animated as a crossfade)
    /// or `reset_highlight` clears it. Color and dim default to the math style's
    /// `.highlight(color, dim)` (yellow, 0.25 without one).
    ///
    /// # Panics
    /// If a name was not defined with `var_color`.
    /// ```ignore
    ///                eq.highlight(["a", "b"]).color(Color::YELLOW).dim(0.2).over(0.5.s()),
    /// ```
    #[must_use]
    pub fn highlight<I>(&self, parts: I) -> crate::transform::MathHighlightBuilder
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let parts: Vec<String> = parts.into_iter().map(|p| p.as_ref().to_string()).collect();
        self.check_parts(&parts);
        crate::transform::MathHighlightBuilder {
            uuid: self.0.id,
            parts,
            color: self.1.0,
            dim: self.1.1,
            duration: 0.5,
            ease: Ease::InOutCubic,
        }
    }

    /// Returns a builder for an animation that removes the highlight and undims everything.
    /// Only `over` and `ease` apply.
    /// ```ignore
    ///                eq.reset_highlight().over(0.5.s()),
    /// ```
    #[must_use]
    pub fn reset_highlight(&self) -> crate::transform::MathHighlightBuilder {
        crate::transform::MathHighlightBuilder {
            uuid: self.0.id,
            parts: Vec::new(),
            color: Color::WHITE,
            dim: 1.0,
            duration: 0.5,
            ease: Ease::InOutCubic,
        }
    }

    fn check_parts<S: AsRef<str>>(&self, names: impl IntoIterator<Item = S>) {
        let AnimObjKind::Math { palette, .. } = &self.0.kind else {
            unreachable!("MathHandle always wraps a Math object")
        };
        for name in names {
            let name = name.as_ref();
            assert!(
                palette.iter().any(|(n, _)| n == name),
                "math part {name:?} is not defined; use var_color. Known parts: {:?}",
                palette.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
            );
        }
    }
}

/// A named part of a math equation, for use in animation builders.
pub struct MathPartHandle {
    uuid: Uuid,
    name: String,
}

impl MathPartHandle {
    /// Returns a builder for an animation that changes this part's color.
    /// ```ignore
    ///                eq.part("a").color().to(Color::RED).over(1.s()),
    /// ```
    #[must_use]
    pub fn color(&self) -> crate::transform::MathPartColorBuilder {
        crate::transform::MathPartColorBuilder {
            uuid: self.uuid,
            part: self.name.clone(),
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }
}
