use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::{AnimOP, IntoAnimOp};
use crate::ease::Ease;
use crate::seconds::Time;

/// The transform of an object, containing its position, rotation, and scale.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    /// Unique identifier for this transform
    pub uuid: Uuid,
    /// Optional parent transform UUID for hierarchical transforms
    pub parent: Option<Uuid>,
    /// 3D position of the object (z is used for draw ordering)
    pub position: Vec3,
    /// Rotation of the object in degrees
    pub rotation: f32,
    /// Uniform or nonuniform scale of the object
    pub scale: Vec2,
}

impl Transform {
    /// Create a new transform at the given position.
    /// A random UUID is generated for identification.
    #[must_use]
    pub fn new(position: Vec3) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            parent: None,
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Set the parent of this transform
    #[must_use]
    pub const fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Returns a builder for an animation that moves this object to a new position.
    /// ```ignore
    ///                transform.position()
    ///                    .to(Vec2::new(100.0, 200.0))
    ///                    .over(5.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub const fn position(&self) -> PositionBuilder {
        PositionBuilder {
            uuid: self.uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    /// Returns a builder for an animation that scales this object.
    /// ```ignore
    ///                transform.scale()
    ///                    .to(Vec2::new(2.0, 2.0))
    ///                    .over(5.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub const fn scale(&self) -> ScaleBuilder {
        ScaleBuilder {
            uuid: self.uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    /// Returns a builder for an animation that rotates this object.
    /// ```ignore
    ///                transform.rotation()
    ///                    .to(360.0)
    ///                    .over(5.s())
    ///                    .ease(Ease::InOutCubic),
    /// ```
    #[must_use]
    pub const fn rotation(&self) -> RotateBuilder {
        RotateBuilder {
            uuid: self.uuid,
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }
}

/// Builder for an animation that moves an object to a target position
pub struct PositionBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<Vec2>,
    pub(crate) object: Option<Uuid>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl PositionBuilder {
    #[must_use]
    /// Move to the position of another object instead of a coordinate
    pub fn object(mut self, target: impl Into<uuid::Uuid>) -> Self {
        self.object = Some(target.into());
        self
    }

    #[must_use]
    /// Set the target position to move to
    pub const fn to(mut self, target: Vec2) -> Self {
        self.target = Some(target);
        self
    }

    #[must_use]
    /// Set the duration of the movement animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    /// Set the easing function and return the animation
    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<PositionBuilder> for AnimOP {
    fn from(b: PositionBuilder) -> Self {
        let target = b.target.unwrap_or(Vec2::ZERO);
        b.object.map_or(
            Self::TransformMovePos(b.uuid, target, b.duration, b.ease, None),
            |obj| Self::TransformMoveToObj(b.uuid, obj, target, b.duration, b.ease, None),
        )
    }
}

impl IntoAnimOp for PositionBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation that scales an object
pub struct ScaleBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<Vec2>,
    pub(crate) object: Option<Uuid>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl ScaleBuilder {
    #[must_use]
    /// Match the scale of another object instead of using a coordinate
    pub fn object(mut self, target: impl Into<uuid::Uuid>) -> Self {
        self.object = Some(target.into());
        self
    }

    #[must_use]
    /// Set the target scale
    pub const fn to(mut self, target: Vec2) -> Self {
        self.target = Some(target);
        self
    }

    #[must_use]
    /// Set the duration of the scale animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    /// Set the easing function and return the animation
    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<ScaleBuilder> for AnimOP {
    fn from(b: ScaleBuilder) -> Self {
        Self::TransformScale(
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

/// Builder for an animation that rotates an object
pub struct RotateBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<f32>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl RotateBuilder {
    #[must_use]
    /// Set the target rotation in degrees
    pub const fn to(mut self, target: f32) -> Self {
        self.target = Some(target);
        self
    }

    #[must_use]
    /// Set the duration of the rotation animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    /// Set the easing function and return the animation
    pub fn ease(mut self, ease: Ease) -> AnimOP {
        self.ease = ease;
        self.into()
    }
}

impl From<RotateBuilder> for AnimOP {
    fn from(b: RotateBuilder) -> Self {
        Self::TransformRotate(b.uuid, b.target.unwrap_or(0.0), b.duration, b.ease, None)
    }
}

impl IntoAnimOp for RotateBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}
