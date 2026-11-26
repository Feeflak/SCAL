use glam::Vec2;
use uuid::Uuid;

use crate::anim_op::{AnimOP, IntoAnimOp};
use crate::ease::Ease;
use crate::seconds::Time;
use crate::transform::{PositionBuilder, RotateBuilder, ScaleBuilder};

/// A duration and an easing curve bundled under one name, so the "feel" of an animation can be
/// defined once and reused with `.with(MOTION)` instead of repeating `.over(..).ease(..)`.
///
/// ```ignore
/// const POP: Motion = Motion::new(0.5, Ease::OutBack);
///
/// timeline![
///     title.scale_to(1.0).with(POP),
///     title.move_to(vec2(0.0, 100.0)).with(POP),
/// ]
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Length of the animation in seconds.
    pub duration: Time,
    /// Easing curve of the animation.
    pub ease: Ease,
}

impl Motion {
    /// What [`crate::AnimObj::pop_in`] uses by default.
    pub const POP_IN: Self = Self::new(0.5, Ease::OutBack);
    /// What [`crate::AnimObj::pop_out`] uses by default.
    pub const POP_OUT: Self = Self::new(0.5, Ease::InBack);

    /// Create a motion from a duration in seconds and an easing curve.
    #[must_use]
    pub const fn new(duration: Time, ease: Ease) -> Self {
        Self { duration, ease }
    }
}

/// Anything that can be used as a scale: a uniform `f32` (`0.5` = half size) or a [`Vec2`].
pub trait IntoScale {
    /// Convert into a per axis scale.
    fn into_scale(self) -> Vec2;
}

impl IntoScale for f32 {
    fn into_scale(self) -> Vec2 {
        Vec2::splat(self)
    }
}

impl IntoScale for Vec2 {
    fn into_scale(self) -> Vec2 {
        self
    }
}

impl IntoScale for [f32; 2] {
    fn into_scale(self) -> Vec2 {
        Vec2::from_array(self)
    }
}

impl IntoScale for (f32, f32) {
    fn into_scale(self) -> Vec2 {
        Vec2::new(self.0, self.1)
    }
}

/// Builders that have a duration and an easing curve. Lets [`GroupOp`] forward `.over()`,
/// `.ease()` and `.with()` to every animation it wraps.
pub trait Tween: Sized {
    /// Set the duration.
    #[must_use]
    fn set_duration(self, duration: Time) -> Self;
    /// Set the easing curve.
    #[must_use]
    fn set_ease(self, ease: Ease) -> Self;
}

/// Several objects that are animated together, created with [`group!`](crate::group).
///
/// Every method returns one parallel animation that takes the usual `.over()`, `.ease()` and
/// `.with()`.
/// ```ignore
/// let on_screen = group![wing, airflow, label];
/// timeline![
///     on_screen.pop_in(),
///     wait(1.s()),
///     on_screen.scale_to(0.0).with(Motion::new(0.3, Ease::InOutBack)),
/// ]
/// ```
#[derive(Clone, Debug)]
pub struct Group {
    ids: Vec<Uuid>,
}

impl Group {
    /// Create a group from transform uuids. Prefer the [`group!`](crate::group) macro.
    #[must_use]
    pub fn new(ids: impl IntoIterator<Item = Uuid>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
        }
    }

    /// Move every object of the group to `target`.
    #[must_use]
    pub fn move_to(&self, target: impl Into<Vec2>) -> GroupOp<PositionBuilder> {
        let target = target.into();
        GroupOp(
            self.ids
                .iter()
                .map(|id| PositionBuilder::new(*id).to(target))
                .collect(),
        )
    }

    /// Scale every object of the group to `target` (an `f32` or a `Vec2`).
    #[must_use]
    pub fn scale_to(&self, target: impl IntoScale) -> GroupOp<ScaleBuilder> {
        let target = target.into_scale();
        GroupOp(
            self.ids
                .iter()
                .map(|id| ScaleBuilder::new(*id).to(target))
                .collect(),
        )
    }

    /// Rotate every object of the group to `degrees`.
    #[must_use]
    pub fn rotate_to(&self, degrees: f32) -> GroupOp<RotateBuilder> {
        GroupOp(
            self.ids
                .iter()
                .map(|id| RotateBuilder::new(*id).to(degrees))
                .collect(),
        )
    }

    /// Scale the group up to full size with [`Motion::POP_IN`].
    #[must_use]
    pub fn pop_in(&self) -> GroupOp<ScaleBuilder> {
        self.scale_to(1.0).with(Motion::POP_IN)
    }

    /// Scale the group down to nothing with [`Motion::POP_OUT`].
    #[must_use]
    pub fn pop_out(&self) -> GroupOp<ScaleBuilder> {
        self.scale_to(0.0).with(Motion::POP_OUT)
    }
}

/// The same animation applied to every object of a [`Group`], played in parallel.
#[derive(Debug)]
pub struct GroupOp<B>(Vec<B>);

impl<B: Tween> GroupOp<B> {
    /// Set the duration of every animation.
    #[must_use]
    pub fn over(self, duration: Time) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|b| b.set_duration(duration))
                .collect(),
        )
    }

    /// Set the easing curve of every animation.
    #[must_use]
    pub fn ease(self, ease: Ease) -> Self {
        Self(self.0.into_iter().map(|b| b.set_ease(ease)).collect())
    }

    /// Set duration and easing from a [`Motion`].
    #[must_use]
    pub fn with(self, motion: Motion) -> Self {
        self.over(motion.duration).ease(motion.ease)
    }
}

impl<B: IntoAnimOp> IntoAnimOp for GroupOp<B> {
    fn into_anim_op(self) -> AnimOP {
        AnimOP::All(
            self.0.into_iter().map(IntoAnimOp::into_anim_op).collect(),
            None,
        )
    }
}

impl<B: IntoAnimOp> From<GroupOp<B>> for AnimOP {
    fn from(op: GroupOp<B>) -> Self {
        op.into_anim_op()
    }
}

/// Group objects so one call animates all of them in parallel. Works with every object that has
/// a `.transform` (all `AnimObj`s and the handles that deref to one).
/// ```ignore
/// let on_screen = group![wing, airflow, label];
/// on_screen.pop_in().with(POP)
/// ```
#[macro_export]
macro_rules! group {
    ( $( $obj:expr ),* $(,)? ) => {
        $crate::Group::new([ $( ($obj).transform.uuid ),* ])
    };
}

/// Instantiate several objects at once. Instantiating takes no time, so this is the same as
/// listing each `.instantiate()` on its own line.
/// ```ignore
/// timeline![
///     spawn![title, subtitle, formula],
/// ]
/// ```
#[macro_export]
macro_rules! spawn {
    ( $( $obj:expr ),* $(,)? ) => {
        $crate::AnimOP::All(vec![ $( ($obj).instantiate() ),* ], None)
    };
}
