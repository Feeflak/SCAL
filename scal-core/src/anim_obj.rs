use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::{AnimOP, CodeAnimationStyle, IntoAnimOp};
use crate::color::Color;
use crate::ease::Ease;
use crate::seconds::Seconds;
use crate::theme::Theme;
use crate::transform::Transform;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimObj {
    pub id: Uuid,
    pub transform: Transform,
    pub kind: AnimObjKind,
}

impl AnimObj {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.clone())
    }
}

pub struct CodeAddLinesBuilder {
    uuid: Uuid,
    text: String,
    from_line: usize,
    duration: Seconds,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeAddLinesBuilder {
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    pub fn from_line(mut self, line: usize) -> Self {
        self.from_line = line;
        self
    }
    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeAddLinesBuilder> for AnimOP {
    fn from(b: CodeAddLinesBuilder) -> AnimOP {
        AnimOP::CodeAddLines(b.uuid, b.text, b.from_line, b.duration, b.ease, b.style)
    }
}

impl IntoAnimOp for CodeAddLinesBuilder {
    fn into_anim_op(self) -> AnimOP { self.into() }
}

pub struct CodeModifyLineBuilder {
    uuid: Uuid,
    line: u32,
    text: String,
    duration: Seconds,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeModifyLineBuilder {
    pub fn str(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeModifyLineBuilder> for AnimOP {
    fn from(b: CodeModifyLineBuilder) -> AnimOP {
        AnimOP::CodeModifyLine(b.uuid, b.line, b.text, b.duration, b.ease, b.style)
    }
}

impl IntoAnimOp for CodeModifyLineBuilder {
    fn into_anim_op(self) -> AnimOP { self.into() }
}

pub struct CodeRemoveLinesBuilder {
    uuid: Uuid,
    range: Range<u32>,
    duration: Seconds,
    ease: Ease,
    style: CodeAnimationStyle,
}

impl CodeRemoveLinesBuilder {
    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    pub fn style(mut self, style: CodeAnimationStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<CodeRemoveLinesBuilder> for AnimOP {
    fn from(b: CodeRemoveLinesBuilder) -> AnimOP {
        AnimOP::CodeRemoveLines(b.uuid, b.range, b.duration, b.ease, b.style)
    }
}

impl IntoAnimOp for CodeRemoveLinesBuilder {
    fn into_anim_op(self) -> AnimOP { self.into() }
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    fn deref(&self) -> &AnimObj { &self.0 }
}

impl CodeHandle {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.0.clone())
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
    fn deref(&self) -> &AnimObj { &self.0 }
}

impl CodeWindowHandle {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.0.clone())
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
}
