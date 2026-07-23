use std::ops::Range;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_builders::{
    CodeAddLinesBuilder, CodeHighlightBuilder, CodeModifyLineBuilder, CodeRemoveLinesBuilder,
    TerminalInputBuilder, TerminalOutputBuilder,
};
use crate::anim_op::{AnimOP, CodeAnimationStyle};
use crate::color::Color;
use crate::ease::Ease;
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash, Eq)]
#[allow(missing_docs)]
pub enum Syntax {
    Rust,
    Nix,
    Python,
    JS,
    Zig,
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
