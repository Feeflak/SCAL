use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::AnimOP;
use crate::ease::Ease;
use crate::seconds::Seconds;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub uuid: Uuid,
    pub parent: Option<Uuid>,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec3) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            parent: None,
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    pub fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn position(&self, target: Vec2) -> PositionBuilder {
        PositionBuilder {
            uuid: self.uuid,
            target,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    pub fn scale(&self, target: Vec2) -> ScaleBuilder {
        ScaleBuilder {
            uuid: self.uuid,
            target,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    pub fn rotate(&self, target: f32) -> RotateBuilder {
        RotateBuilder {
            uuid: self.uuid,
            target,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }
}

pub struct PositionBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Vec2,
    pub(crate) object: Option<Uuid>,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl PositionBuilder {
    pub fn object(mut self, target: uuid::Uuid) -> Self {
        self.object = Some(target);
        self
    }

    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }

    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<PositionBuilder> for AnimOP {
    fn from(b: PositionBuilder) -> AnimOP {
        match b.object {
            Some(target) => AnimOP::TransformMoveToObj(b.uuid, target, b.target, b.duration, b.ease),
            None => AnimOP::TransformMovePos(b.uuid, b.target, b.duration, b.ease),
        }
    }
}

pub struct ScaleBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Vec2,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl ScaleBuilder {
    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }

    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<ScaleBuilder> for AnimOP {
    fn from(b: ScaleBuilder) -> AnimOP {
        AnimOP::TransformScale(b.uuid, b.target, b.duration, b.ease)
    }
}

pub struct RotateBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: f32,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl RotateBuilder {
    pub fn over(mut self, duration: Seconds) -> Self {
        self.duration = duration;
        self
    }

    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<RotateBuilder> for AnimOP {
    fn from(b: RotateBuilder) -> AnimOP {
        AnimOP::TransformRotate(b.uuid, b.target, b.duration, b.ease)
    }
}
