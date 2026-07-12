use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_builders::{
    CodeAddLinesBuilder, CodeHighlightBuilder, CodeModifyLineBuilder, CodeRemoveLinesBuilder,
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
    /// ```
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
        alignment: TextAlign,
        color: Color,
        font_size: f32,
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
    Group {
        children: Vec<AnimObj>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextAlign {
    Center,
    Left,
    Right,
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
    /// ```
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
    /// ```
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
    /// ```
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
        }
    }

    /// Returns a builder for an animation of removing lines form a code block.
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
        }
    }

    /// Returns a builder for an animation of removing lines form a code block.
    /// ```
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

macro_rules! impl_handle {
    ($type:ty) => {
        impl $type {
            /// Returns a builder for an animation that moves this object to a target position.
            /// ```
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
            /// ```
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
            /// ```
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
