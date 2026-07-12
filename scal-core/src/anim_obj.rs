use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::{AnimOP, CodeAnimationStyle, IntoAnimOp};
use crate::color::Color;
use crate::ease::Ease;
use crate::seconds::Time;
use crate::theme::Theme;
use crate::transform::Transform;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimObj {
    pub id: Uuid,
    pub transform: Transform,
    pub kind: AnimObjKind,
}

impl AnimObj {
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.clone(), None)
    }
}

pub struct CodeAddLinesBuilder {
    uuid: Uuid,
    text: String,
    from_line: usize,
    duration: Time,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeAddLinesBuilder {
    #[must_use]
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    #[must_use]
    pub fn from_line(mut self, line: usize) -> Self {
        self.from_line = line;
        self
    }
    #[must_use]
    pub fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeAddLinesBuilder> for AnimOP {
    fn from(b: CodeAddLinesBuilder) -> AnimOP {
        AnimOP::CodeAddLines(
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

pub struct CodeModifyLineBuilder {
    uuid: Uuid,
    line: u32,
    text: String,
    duration: Time,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeModifyLineBuilder {
    #[must_use]
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    #[must_use]
    pub fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeModifyLineBuilder> for AnimOP {
    fn from(b: CodeModifyLineBuilder) -> AnimOP {
        AnimOP::CodeModifyLine(b.uuid, b.line, b.text, b.duration, b.ease, b.style, None)
    }
}

impl IntoAnimOp for CodeModifyLineBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

pub struct CodeRemoveLinesBuilder {
    uuid: Uuid,
    range: Range<u32>,
    duration: Time,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeRemoveLinesBuilder {
    #[must_use]
    pub fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    #[must_use]
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    #[must_use]
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeRemoveLinesBuilder> for AnimOP {
    fn from(b: CodeRemoveLinesBuilder) -> AnimOP {
        AnimOP::CodeRemoveLines(b.uuid, b.range, b.duration, b.ease, b.style, None)
    }
}

impl IntoAnimOp for CodeRemoveLinesBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TextAlign {
    Center,
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum Syntax {
    Rust,
    Nix,
    Python,
    JS,
    Zig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StretchMode {
    Fit,
    Fill,
}

pub struct CodeHandle(pub AnimObj);

impl std::ops::Deref for CodeHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl CodeHandle {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.0.clone(), None)
    }
    pub fn add_lines(&self) -> CodeAddLinesBuilder {
        CodeAddLinesBuilder {
            uuid: self.0.id,
            text: String::new(),
            from_line: 0,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }
    pub fn modify_line(&self, line: u32) -> CodeModifyLineBuilder {
        CodeModifyLineBuilder {
            uuid: self.0.id,
            line,
            text: String::new(),
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }
    pub fn remove_lines(&self, range: Range<u32>) -> CodeRemoveLinesBuilder {
        CodeRemoveLinesBuilder {
            uuid: self.0.id,
            range,
            duration: 1.0,
            ease: Ease::Linear,
            style: CodeAnimationStyle::TypeWriter,
        }
    }
}

pub struct CodeWindowHandle(pub AnimObj);

impl std::ops::Deref for CodeWindowHandle {
    type Target = AnimObj;
    fn deref(&self) -> &AnimObj {
        &self.0
    }
}

impl CodeWindowHandle {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.0.clone(), None)
    }
    pub fn add_lines(&self) -> CodeAddLinesBuilder {
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
    pub fn modify_line(&self, line: u32) -> CodeModifyLineBuilder {
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
    pub fn remove_lines(&self, range: Range<u32>) -> CodeRemoveLinesBuilder {
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


        if let AnimObjKind::CodeWindow { close_btn_id, .. } = &self.0.kind {
            CircleHandle(*close_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
        if let AnimObjKind::CodeWindow {
            minimize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*minimize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
        if let AnimObjKind::CodeWindow {
            maximize_btn_id, ..
        } = &self.0.kind
        {
            CircleHandle(*maximize_btn_id)
        } else {
            CircleHandle(self.0.id)
        }
    }
    pub const fn title_text(&self) -> TextHandle {
        if let AnimObjKind::CodeWindow { title_id, .. } = &self.0.kind {
            TextHandle(*title_id)
        } else {
            TextHandle(self.0.id)
        }
    }
    pub const fn window_background(&self) -> RectangleHandle {
        RectangleHandle(self.0.id)
    }
    pub const fn container(&self) -> RectangleHandle {
        if let AnimObjKind::CodeWindow { container_id, .. } = &self.0.kind {
            RectangleHandle(*container_id)
        } else {
            RectangleHandle(self.0.id)
        }
    }
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
            pub fn position(&self) -> crate::transform::PositionBuilder {
                crate::transform::PositionBuilder {
                    uuid: self.0,
                    target: None,
                    object: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
            pub fn scale(&self) -> crate::transform::ScaleBuilder {
                crate::transform::ScaleBuilder {
                    uuid: self.0,
                    target: None,
                    object: None,
                    duration: 1.0,
                    ease: Ease::Linear,
                }
            }
            pub fn rotation(&self) -> crate::transform::RotateBuilder {
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
