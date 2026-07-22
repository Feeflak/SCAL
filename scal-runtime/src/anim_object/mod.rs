use super::types::*;
use crate::anim_op::AnimOperation;
use anyhow::{Context, Result};
use glam::{Vec2, Vec3};
use scal_core::CodeAnimationStyle;
use scal_core::Color;
use scal_core::Ease;
use scal_core::StretchMode;
use scal_core::Syntax;
use scal_core::Theme;
use uuid::Uuid;

pub mod code_window;
pub mod compose;
pub mod image;
pub mod object_trait;
pub mod primitive_shapes;
pub mod render;
pub mod svg;
pub mod terminal;
pub mod text;

use object_trait::DynAnimObj;

pub use self::code_window::CodeWindow;
pub use self::code_window::code_window;
pub use self::compose::{
    Alignment, LayoutBackground, LayoutContainer, LayoutDir, LayoutItem, LayoutResult, PinPoint,
    layout,
};
use self::image::Image;
use self::primitive_shapes::{Circle, Polygon, Rectangle};
use self::svg::Svg;
use self::text::{Align, Text, code::Code, code::TextLine};

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
    padding: f32,
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
        padding,
        show_line_numbers: false,
        line_number_color: Color::new(0.5, 0.5, 0.5, 0.6),
        anim_reveal: 1.0,
        anim_spacing: 0.0,
        anim_line_start: 0,
        anim_line_end: 0,
        anim_style: CodeAnimationStyle::TypeWriter,
        anim_spacing_accum: 0.0,
        cached_size: None,
        highlights: vec![],
    }
}

pub fn text(
    transform: Transform,
    value: String,
    font_family: String,
    alignment: Align,
    color: Color,
    font_size: f32,
    modifications: Vec<scal_core::TextModifier>,
) -> DynAnimObj {
    DynAnimObj(Box::new(Text {
        id: transform.uuid,
        font_family,
        alignment,
        value,
        color,
        font_size,
        transform,
        cached_size: None,
        modifications,
    }))
}

pub fn rectangle(transform: Transform, size: Vec2, corner_radius: f32, color: Color) -> DynAnimObj {
    DynAnimObj(Box::new(Rectangle {
        size,
        corner_radius,
        color,
        transform,
    }))
}

pub fn circle(transform: Transform, radius: f32, color: Color) -> DynAnimObj {
    DynAnimObj(Box::new(Circle {
        radius,
        color,
        transform,
    }))
}

pub fn polygon(transform: Transform, radius: f32, sides: u32, color: Color) -> DynAnimObj {
    DynAnimObj(Box::new(Polygon {
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
) -> DynAnimObj {
    DynAnimObj(Box::new(Image {
        path,
        size,
        color,
        stretch,
        transform,
    }))
}

pub fn svg(
    transform: Transform,
    path: &str,
    size: Vec2,
    tint: Color,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: Option<f32>,
    stretch: StretchMode,
) -> DynAnimObj {
    DynAnimObj(Box::new(Svg {
        path: path.to_string(),
        size,
        tint,
        fill,
        stroke,
        stroke_width,
        stretch,
        transform,
    }))
}

pub fn wait(time: Seconds) -> AnimOperation {
    AnimOperation::Wait(time, None)
}
impl From<Vec<AnimOperation>> for AnimOperation {
    fn from(value: Vec<AnimOperation>) -> Self {
        AnimOperation::All(value, None)
    }
}
#[derive(Clone, Debug, Copy)]
pub struct Transform {
    pub(crate) scale: Vec2,
    pub uuid: Uuid,
    pub parent: Option<Uuid>,
    pub(crate) position: Vec3,
    pub(crate) rotation: f32,
    pub layout_container: Option<Uuid>,
    pub(crate) world_uniform: Option<TransformUniform>,
}
#[derive(Clone, Debug, Copy)]
pub struct TransformUniform {
    pub scale: Vec2,
    pub position: Vec3,
    pub rotation: f32,
}
impl Transform {
    pub fn get_world_uniform(&self) -> Result<TransformUniform> {
        self.world_uniform
            .context("world uniform was not cached; call get_object_world_matrix first")
    }
    pub fn set_parent(&mut self, parent: Option<Uuid>) {
        self.parent = parent;
        self.world_uniform = None;
    }
    pub fn position_to(&self, to: Vec2, time: Seconds, curve: Ease) -> AnimOperation {
        AnimOperation::TransformMovePos(self.uuid, to, time, curve, None)
    }
    pub fn position_to_object(
        &self,
        target: &DynAnimObj,
        offset: Vec2,
        time: Seconds,
        curve: Ease,
    ) -> AnimOperation {
        AnimOperation::TransformMoveToObj(self.uuid, target.uuid(), offset, time, curve, None)
    }
    pub fn rotate_to(&self, to: f32, time: Seconds, curve: Ease) -> AnimOperation {
        AnimOperation::TransformRotate(self.uuid, to, time, curve, None)
    }
    pub fn scale_to(&self, to: Vec2, time: Seconds, curve: Ease) -> AnimOperation {
        AnimOperation::TransformScale(self.uuid, to, time, curve, None)
    }
}

impl Transform {
    pub fn new(parent: Option<&DynAnimObj>, position: Vec3, rotation: f32, scale: Vec2) -> Self {
        let parent_uuid = parent.map(|obj| obj.uuid());
        Self {
            rotation,
            uuid: Uuid::new_v4(),
            parent: parent_uuid,
            position,
            scale,
            layout_container: None,
            world_uniform: None,
        }
    }
    pub fn with_uuid(uuid: Uuid, position: Vec3) -> Self {
        Self {
            uuid,
            parent: None,
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
            layout_container: None,
            world_uniform: None,
        }
    }
}
