use super::types::*;
use anim_op::AnimOP;
use glam::{Vec2, Vec3};
use uuid::Uuid;

use crate::anim_op::{self, AnimationCurve};

pub mod code_window;
pub mod compose;
pub mod image;
pub mod object_trait;
pub mod primitive_shapes;
pub mod render;
pub mod svg;
pub mod text;

use object_trait::AnimObj;

pub use self::code_window::CodeWindow;
pub use self::code_window::code_window;
pub use self::compose::{
    LayoutResult, LayoutContainer, LayoutDir, Alignment, PinPoint, LayoutBackground, LayoutItem, layout,
};
use self::image::{Image, StretchMode};
use self::primitive_shapes::{Circle, Polygon, Rectangle};
use self::svg::Svg;
use self::text::{code::Code, code::CodeAnimationStyle, code::TextLine, Align, Text};

use self::text::code::Syntax;
use self::text::code::theme::Theme;

pub fn transform(position: Vec3) -> Transform {
    Transform::new(None, position, 0., Vec2::ONE)
}

pub fn code(
    transform: Transform,
    source_code: String,
    theme: Theme,
    font_family: String,
    alignment: Align,
    font_size: f32,
    syntax: Syntax,
    lines: Vec<TextLine>,
) -> Code {
    Code {
        id: transform.uuid,
        transform,
        source_code,
        theme,
        font_family,
        alignment,
        font_size,
        syntax,
        lines,
        dirty: true,
        anim_reveal: 1.0,
        anim_spacing: 0.0,
        anim_line_start: 0,
        anim_line_end: 0,
        anim_style: CodeAnimationStyle::TypeWriter,
        anim_spacing_accum: 0.0,
    }
}

pub fn text(
    transform: Transform,
    value: String,
    font_family: String,
    alignment: Align,
    color: Color,
    font_size: f32,
) -> AnimObj {
    AnimObj(Box::new(Text {
        id: transform.uuid,
        font_family,
        alignment,
        value,
        color,
        font_size,
        transform,
    }))
}

pub fn rectangle(
    transform: Transform,
    size: Vec2,
    corner_radius: f32,
    color: Color,
) -> AnimObj {
    AnimObj(Box::new(Rectangle {
        size,
        corner_radius,
        color,
        transform,
    }))
}

pub fn circle(
    transform: Transform,
    radius: f32,
    color: Color,
) -> AnimObj {
    AnimObj(Box::new(Circle {
        radius,
        color,
        transform,
    }))
}

pub fn polygon(
    transform: Transform,
    radius: f32,
    sides: u32,
    color: Color,
) -> AnimObj {
    AnimObj(Box::new(Polygon {
        radius,
        sides,
        color,
        transform,
    }))
}

pub fn image(
    transform: Transform,
    path: String,
    size: Vec2,
    color: Color,
    stretch: StretchMode,
) -> AnimObj {
    AnimObj(Box::new(Image {
        path,
        size,
        color,
        stretch,
        transform,
    }))
}

pub fn svg(
    transform: Transform,
    path: String,
    size: Vec2,
    tint: Color,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: Option<f32>,
    stretch: StretchMode,
) -> AnimObj {
    AnimObj(Box::new(Svg {
        path,
        size,
        tint,
        fill,
        stroke,
        stroke_width,
        stretch,
        transform,
    }))
}

pub fn wait(time: Seconds) -> AnimOP {
    AnimOP::Wait(time)
}
impl From<Vec<AnimOP>> for AnimOP {
    fn from(value: Vec<AnimOP>) -> Self {
        AnimOP::All(value)
    }
}
#[derive(Clone, Debug)]
pub struct Transform {
    pub scale: Vec2,
    pub uuid: Uuid,
    pub parent: Option<Uuid>,
    pub position: Vec3,
    pub rotation: f32,
    pub layout_container: Option<Uuid>,
}
impl Transform {
    pub fn position_to(&self, to: Vec2, time: Seconds, curve: AnimationCurve) -> AnimOP {
        AnimOP::TransformMovePos(self.uuid, to, time, curve)
    }
    pub fn rotate_to(&self, to: f32, time: Seconds, curve: AnimationCurve) -> AnimOP {
        AnimOP::TransformRotate(self.uuid, to, time, curve)
    }
    pub fn scale_to(&self, to: Vec2, time: Seconds, curve: AnimationCurve) -> AnimOP {
        AnimOP::TransformScale(self.uuid, to, time, curve)
    }
}

impl Transform {
    pub fn new(parent: Option<&AnimObj>, position: Vec3, rotation: f32, scale: Vec2) -> Self {
        Self {
            rotation,
            uuid: Uuid::new_v4(),
            parent: parent.map(|obj| obj.uuid()),
            position,
            scale,
            layout_container: None,
        }
    }
}
