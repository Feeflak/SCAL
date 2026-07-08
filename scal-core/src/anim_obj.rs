use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::color::Color;
use crate::transform::Transform;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimObj {
    pub id: Uuid,
    pub transform: Transform,
    pub kind: AnimObjKind,
}

impl AnimObj {
    pub fn instantiate(&self) -> crate::anim_op::AnimOP {
        crate::anim_op::AnimOP::Instantiate(self.clone())
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
        theme: Vec<u32>,
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
