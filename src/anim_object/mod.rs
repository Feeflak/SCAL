use super::types::*;
use anim_op::AnimOP;
use glam::{Vec2, Vec3};
use uuid::Uuid;

use crate::anim_op::{self, AnimationCurve};

pub mod primitive_shapes;
pub mod render;
pub mod text;

impl PartialEq for AnimObject {
    fn eq(&self, other: &Self) -> bool {
        self.transform().uuid == other.transform().uuid
    }
}
#[derive(Clone, Debug)]

pub enum AnimObject {
    Code(text::code::Code, Transform),
    Text(text::Text, Transform),
    Square(primitive_shapes::Rectangle, Transform),
    Circle(primitive_shapes::Circle, Transform),
    Polygon(primitive_shapes::Polygon, Transform),
}
impl AnimObject {
    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            AnimObject::Code(_, t) => t,
            AnimObject::Text(_, t) => t,
            AnimObject::Square(_, t) => t,
            AnimObject::Circle(_, t) => t,
            AnimObject::Polygon(_, t) => t,
        }
    }
    pub fn transform(&self) -> &Transform {
        match self {
            AnimObject::Code(_, t) => t,
            AnimObject::Text(_, t) => t,
            AnimObject::Square(_, t) => t,
            AnimObject::Circle(_, t) => t,
            AnimObject::Polygon(_, t) => t,
        }
    }
}
impl AnimObject {
    pub fn instantiate(&self) -> anim_op::AnimOP {
        AnimOP::Instantiate(self.clone())
    }
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
    pub pos: Vec3,
    pub rotation: f32,
}
impl Transform {
    pub fn move_local(&self, to: Vec2, time: Seconds, curve: AnimationCurve) -> AnimOP {
        AnimOP::TransformMovePos(self.uuid, to, time, curve)
    }
    pub fn add_children(&self, children: Vec<AnimObject>) -> AnimOP {
        todo!()
    }
}

impl Transform {
    pub fn new(parent: Option<&AnimObject>, pos: Vec3, rotation: f32, scale: Vec2) -> Self {
        Self {
            rotation,
            uuid: Uuid::new_v4(),
            parent: parent.map(|obj| obj.transform().uuid),
            pos,
            scale,
        }
    }
}
