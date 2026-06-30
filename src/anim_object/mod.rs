use super::types::*;
use anim_op::AnimOP;
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
    Text(text::Text, Transform),
    Square(primitive_shapes::Square, Transform),
}
impl AnimObject {
    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            AnimObject::Text(_, t) => t,
            AnimObject::Square(_, t) => t,
        }
    }
    pub fn transform(&self) -> &Transform {
        match self {
            AnimObject::Text(_, t) => t,
            AnimObject::Square(_, t) => t,
        }
    }
}
impl AnimObject {
    pub fn instantiate(self) -> anim_op::AnimOP {
        AnimOP::Instantiate(self)
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
    pub scale: f32,
    pub uuid: Uuid,
    pub children: Vec<Uuid>,
    pub pos: Vec2,
    pub rotation: f32,
    pub z: f32,
}
impl Transform {
    pub fn move_local(self, to: Vec2, time: Seconds, curve: AnimationCurve) -> AnimOP {
        AnimOP::TransformMovePos(self.uuid, to, time, curve)
    }
    pub fn add_children(self, children: Vec<AnimObject>) -> AnimOP {
        todo!()
    }
}

impl Transform {
    pub fn new(children: Vec<Uuid>, pos: Vec2, rotation: f32, scale: f32, z: f32) -> Self {
        Self {
            rotation,
            uuid: Uuid::new_v4(),
            children,
            pos,
            scale,
            z,
        }
    }
}
