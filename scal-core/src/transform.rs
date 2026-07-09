use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::{AnimOP, IntoAnimOp};
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

    pub fn position(&self) -> PositionBuilder {
        PositionBuilder {
            uuid: self.uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    pub fn scale(&self) -> ScaleBuilder {
        ScaleBuilder {
            uuid: self.uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    pub fn rotation(&self) -> RotateBuilder {
        RotateBuilder {
            uuid: self.uuid,
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }
}

pub struct PositionBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<Vec2>,
    pub(crate) object: Option<Uuid>,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl PositionBuilder {
    pub fn object(mut self, target: impl Into<uuid::Uuid>) -> Self {
        self.object = Some(target.into());
        self
    }

    pub fn to(mut self, target: Vec2) -> Self {
        self.target = Some(target);
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
        let target = b.target.unwrap_or(Vec2::ZERO);
        match b.object {
            Some(obj) => AnimOP::TransformMoveToObj(b.uuid, obj, target, b.duration, b.ease, None),
            None => AnimOP::TransformMovePos(b.uuid, target, b.duration, b.ease, None),
        }
    }
}

impl IntoAnimOp for PositionBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

pub struct ScaleBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<Vec2>,
    pub(crate) object: Option<Uuid>,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl ScaleBuilder {
    pub fn object(mut self, target: impl Into<uuid::Uuid>) -> Self {
        self.object = Some(target.into());
        self
    }

    pub fn to(mut self, target: Vec2) -> Self {
        self.target = Some(target);
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

impl From<ScaleBuilder> for AnimOP {
    fn from(b: ScaleBuilder) -> AnimOP {
        AnimOP::TransformScale(
            b.uuid,
            b.target.unwrap_or(Vec2::ONE),
            b.duration,
            b.ease,
            None,
        )
    }
}

impl IntoAnimOp for ScaleBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

pub struct RotateBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<f32>,
    pub(crate) duration: Seconds,
    pub(crate) ease: Ease,
}

impl RotateBuilder {
    pub fn to(mut self, target: f32) -> Self {
        self.target = Some(target);
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

impl From<RotateBuilder> for AnimOP {
    fn from(b: RotateBuilder) -> AnimOP {
        AnimOP::TransformRotate(b.uuid, b.target.unwrap_or(0.0), b.duration, b.ease, None)
    }
}

impl IntoAnimOp for RotateBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}
