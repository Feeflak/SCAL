use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::anim_op::{AnimOP, IntoAnimOp};
use crate::color::Color;
use crate::ease::Ease;
use crate::motion::{Motion, Tween};
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
    /// Point the object rotates around, as an offset in pixels from the object's center
    /// (y points down). `Vec2::ZERO` rotates around the center. Setting a pivot never moves
    /// the object, it only changes what `rotation` turns it around.
    pub pivot: Vec2,
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
            pivot: Vec2::ZERO,
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
        PositionBuilder::new(self.uuid)
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
        ScaleBuilder::new(self.uuid)
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
        RotateBuilder::new(self.uuid)
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
    pub(crate) const fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

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
    /// Set the easing function and return the builder
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
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
    pub(crate) const fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            target: None,
            object: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

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
    /// Set the easing function and return the builder
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
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
    pub(crate) const fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            target: None,
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

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
    /// Set the easing function and return the builder
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
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

/// Builder for an animation that changes an object's color
pub struct ColorBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) target: Option<Color>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl ColorBuilder {
    /// Set the target color
    pub const fn to(mut self, target: Color) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the duration of the color animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function and return the builder
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<ColorBuilder> for AnimOP {
    fn from(b: ColorBuilder) -> Self {
        Self::ObjectColor(
            b.uuid,
            b.target.unwrap_or(Color::WHITE),
            b.duration,
            b.ease,
            None,
        )
    }
}

impl IntoAnimOp for ColorBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation that changes the color of a named part of a math object
pub struct MathPartColorBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) part: String,
    pub(crate) target: Option<Color>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl MathPartColorBuilder {
    /// Set the target color
    pub const fn to(mut self, target: Color) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the duration of the color animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<MathPartColorBuilder> for AnimOP {
    fn from(b: MathPartColorBuilder) -> Self {
        Self::MathPartColor(
            b.uuid,
            b.part,
            b.target.unwrap_or(Color::WHITE),
            b.duration,
            b.ease,
            None,
        )
    }
}

impl IntoAnimOp for MathPartColorBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation that highlights parts of a math object and dims everything else.
/// The highlight stays until another highlight or a reset replaces it.
pub struct MathHighlightBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) parts: Vec<String>,
    pub(crate) color: Color,
    pub(crate) dim: f32,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl MathHighlightBuilder {
    /// Color the highlighted parts are tinted to (default: yellow)
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Alpha everything that is not highlighted fades to (default 0.25, 1.0 = no dimming)
    pub const fn dim(mut self, alpha: f32) -> Self {
        self.dim = alpha;
        self
    }

    /// Set the duration of the transition
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<MathHighlightBuilder> for AnimOP {
    fn from(b: MathHighlightBuilder) -> Self {
        Self::MathHighlight(b.uuid, b.parts, b.color, b.dim, b.duration, b.ease, None)
    }
}

impl IntoAnimOp for MathHighlightBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation that counts the number shown by a text or math object from one
/// value to another. The object's `{:spec}` placeholders (see [`crate::number_format`]) are
/// re-rendered every frame.
pub struct NumberBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) from: Option<f32>,
    pub(crate) target: Option<f32>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl NumberBuilder {
    #[must_use]
    /// Set the number the animation starts from. By default it starts from whatever number the
    /// object is showing when the animation begins.
    pub const fn from(mut self, start: f32) -> Self {
        self.from = Some(start);
        self
    }

    #[must_use]
    /// Set the number the animation ends on
    pub const fn to(mut self, target: f32) -> Self {
        self.target = Some(target);
        self
    }

    #[must_use]
    /// Set the duration of the animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    /// Set the easing function and return the builder
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<NumberBuilder> for AnimOP {
    fn from(b: NumberBuilder) -> Self {
        Self::NumberTo(
            b.uuid,
            b.from,
            b.target.or(b.from).unwrap_or(0.0),
            b.duration,
            b.ease,
            None,
        )
    }
}

impl IntoAnimOp for NumberBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

/// Builder for an animation that moves points of a vector to new scene positions, see
/// [`crate::VectorHandle::move_point`].
pub struct VectorPointsBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) point_count: usize,
    pub(crate) moves: Vec<(usize, Vec2)>,
    pub(crate) duration: Time,
    pub(crate) ease: Ease,
}

#[allow(clippy::return_self_not_must_use)]
impl VectorPointsBuilder {
    pub(crate) const fn new(uuid: Uuid, point_count: usize) -> Self {
        Self {
            uuid,
            point_count,
            moves: Vec::new(),
            duration: 1.0,
            ease: Ease::Linear,
        }
    }

    #[must_use]
    /// Also move point `index` to `target`, in the same animation. Moving the same point twice
    /// keeps the last target.
    ///
    /// # Panics
    /// If the vector has no point `index`.
    pub fn point(mut self, index: usize, target: impl Into<Vec2>) -> Self {
        assert!(
            index < self.point_count,
            "the vector has {} points, there is no point {index}",
            self.point_count
        );
        let target = target.into();
        match self.moves.iter_mut().find(|(i, _)| *i == index) {
            Some(existing) => existing.1 = target,
            None => self.moves.push((index, target)),
        }
        self
    }

    #[must_use]
    /// Set the duration of the animation
    pub const fn over(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    /// Set the easing function
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}

impl From<VectorPointsBuilder> for AnimOP {
    fn from(b: VectorPointsBuilder) -> Self {
        Self::VectorMovePoints(b.uuid, b.moves, b.duration, b.ease, None)
    }
}

impl IntoAnimOp for VectorPointsBuilder {
    fn into_anim_op(self) -> AnimOP {
        self.into()
    }
}

macro_rules! impl_motion {
    ($($builder:ty),* $(,)?) => {$(
        #[allow(clippy::return_self_not_must_use)]
        impl $builder {
            /// Set duration and easing at once from a named [`Motion`], instead of
            /// `.over(..).ease(..)`.
            /// ```ignore
            /// const POP: Motion = Motion::new(0.5, Ease::OutBack);
            /// title.scale_to(1.0).with(POP)
            /// ```
            #[must_use]
            pub const fn with(mut self, motion: Motion) -> Self {
                self.duration = motion.duration;
                self.ease = motion.ease;
                self
            }
        }

        impl Tween for $builder {
            fn set_duration(mut self, duration: Time) -> Self {
                self.duration = duration;
                self
            }
            fn set_ease(mut self, ease: Ease) -> Self {
                self.ease = ease;
                self
            }
        }
    )*};
}

impl_motion!(
    PositionBuilder,
    ScaleBuilder,
    RotateBuilder,
    ColorBuilder,
    MathPartColorBuilder,
    MathHighlightBuilder,
    NumberBuilder,
    VectorPointsBuilder,
);
