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
    /// One instantiate op per object, in the order they were grouped.
    spawn_ops: Vec<AnimOP>,
}

impl Group {
    /// Create a group from transform uuids. Prefer the [`group!`](crate::group) macro.
    #[must_use]
    pub fn new(ids: impl IntoIterator<Item = Uuid>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            spawn_ops: vec![],
        }
    }

    /// Create a group that can also be spawned: `spawn_ops` are the `instantiate()` ops of the
    /// objects. Prefer the [`group!`](crate::group) macro.
    #[doc(hidden)]
    #[must_use]
    pub const fn with_spawn_ops(ids: Vec<Uuid>, spawn_ops: Vec<AnimOP>) -> Self {
        Self { ids, spawn_ops }
    }

    /// Add every object of the group to the scene at once. This is what `spawn![group]` uses.
    /// Groups made with [`Group::new`] have no objects to add, only the `group!` macro keeps them.
    #[must_use]
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::All(self.spawn_ops.clone(), None)
    }

    /// Remove every object of the group from the scene for good, see [`drop!`](crate::drop).
    /// This is what `drop![group]` uses.
    #[must_use]
    pub fn despawn(&self) -> AnimOP {
        AnimOP::All(
            self.ids
                .iter()
                .map(|id| AnimOP::Drop(*id, None))
                .collect(),
            None,
        )
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
        {
            #[allow(unused_mut)]
            let mut ids = ::std::vec::Vec::new();
            #[allow(unused_mut)]
            let mut spawn_ops = ::std::vec::Vec::new();
            $(
                {
                    // Evaluate each object expression only once.
                    let object = &($obj);
                    ids.push(object.transform.uuid);
                    spawn_ops.push(object.instantiate());
                }
            )*
            $crate::Group::with_spawn_ops(ids, spawn_ops)
        }
    };
}

/// Instantiate several objects at once. Instantiating takes no time, so this is the same as
/// listing each `.instantiate()` on its own line. Takes objects and [`group!`](crate::group)s,
/// mixed as you like.
/// ```ignore
/// let labels = group![title, subtitle];
/// timeline![
///     spawn![labels, formula],
/// ]
/// ```
#[macro_export]
macro_rules! spawn {
    ( $( $obj:expr ),* $(,)? ) => {
        $crate::AnimOP::All(vec![ $( ($obj).instantiate() ),* ], None)
    };
}

/// Remove objects from the scene for good, in zero time. Anything attached to an object (the
/// parts of a code window, the children of a layout) goes with it, and the memory they used
/// (glyph atlas space, mesh data) is freed.
///
/// Objects are never removed automatically. After `pop_out` an object is only scaled to nothing
/// and still costs memory, so drop what a later part of the video does not need anymore.
/// Takes objects (anything with a `.transform`) and [`group!`](crate::group)s, mixed as you like.
/// ```ignore
/// let labels = group![title, subtitle];
/// sequence![
///     parallel![cw.pop_out().with(POP), labels.pop_out()],
///     drop![cw, labels],
/// ]
/// ```
#[macro_export]
macro_rules! drop {
    ( $( $obj:expr ),* $(,)? ) => {
        $crate::AnimOP::All(vec![ $( ($obj).despawn() ),* ], None)
    };
}

#[cfg(test)]
mod drop_tests {
    use crate::{AnimOP, prelude::*};

    #[test]
    fn drop_macro_drops_every_object() {
        let a = rectangle().build();
        let b = text().build();
        let AnimOP::All(ops, _) = drop![a, b,] else {
            panic!("drop! should be a parallel group");
        };
        assert!(matches!(ops[0], AnimOP::Drop(id, _) if id == a.transform.uuid));
        assert!(matches!(ops[1], AnimOP::Drop(id, _) if id == b.transform.uuid));
        assert_eq!(ops.len(), 2);
    }
}

#[cfg(test)]
mod group_spawn_tests {
    use crate::{AnimOP, prelude::*};

    #[test]
    fn spawn_and_drop_accept_groups() {
        let a = rectangle().build();
        let b = text().build();
        let c = rectangle().build();
        let pair = group![a, b];

        let AnimOP::All(spawned, _) = spawn![pair, c] else {
            panic!("spawn! should be a parallel group");
        };
        // The group spawns as one nested op holding both objects.
        let AnimOP::All(inner, _) = &spawned[0] else {
            panic!("a group should spawn as one op");
        };
        assert_eq!(inner.len(), 2);
        assert!(matches!(&inner[0], AnimOP::Instantiate(o, _) if o.transform.uuid == a.transform.uuid));
        assert!(matches!(&spawned[1], AnimOP::Instantiate(o, _) if o.transform.uuid == c.transform.uuid));

        let AnimOP::All(dropped, _) = drop![pair, c] else {
            panic!("drop! should be a parallel group");
        };
        let AnimOP::All(inner, _) = &dropped[0] else {
            panic!("a group should drop as one op");
        };
        assert!(matches!(inner[1], AnimOP::Drop(id, _) if id == b.transform.uuid));
        assert!(matches!(dropped[1], AnimOP::Drop(id, _) if id == c.transform.uuid));
    }

    #[test]
    fn group_expressions_are_evaluated_once() {
        let g = group![rectangle().build()];
        let AnimOP::All(spawned, _) = g.instantiate() else {
            panic!()
        };
        let AnimOP::All(dropped, _) = g.despawn() else {
            panic!()
        };
        let (AnimOP::Instantiate(o, _), AnimOP::Drop(id, _)) = (&spawned[0], &dropped[0]) else {
            panic!()
        };
        assert_eq!(o.transform.uuid, *id);
    }
}
