pub mod code;
mod transform;

use anyhow::{Context, Result};
use glam::{Vec2, Vec4Swizzles, vec3};
use log::debug;
use uuid::Uuid;

use crate::anim_object::TransformUniform;
use crate::anim_object::object_trait::AnimObj;
use crate::animator::Animator;
use crate::types::{Seconds, Sfx};

#[derive(Clone)]
pub struct CurrentClosure(pub std::sync::Arc<dyn Fn(AnimObj) -> AnimOP + Send + Sync>);
impl std::fmt::Debug for CurrentClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentClosure").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub enum AnimOP {
    Instantiate(AnimObj, Option<scal_core::SourceLoc>),
    TransformMovePos(Uuid, Vec2, Seconds, AnimationCurve, Option<scal_core::SourceLoc>),
    TransformMoveToObj(Uuid, Uuid, Vec2, Seconds, AnimationCurve, Option<scal_core::SourceLoc>),
    TransformRotate(Uuid, f32, Seconds, AnimationCurve, Option<scal_core::SourceLoc>),
    TransformScale(Uuid, Vec2, Seconds, AnimationCurve, Option<scal_core::SourceLoc>),

    CodeAddLines(
        Uuid,
        String,
        usize,
        Seconds,
        AnimationCurve,
        crate::anim_object::text::code::CodeAnimationStyle,
        Option<scal_core::SourceLoc>,
    ),
    CodeModifyLine(
        Uuid,
        u32,
        String,
        Seconds,
        AnimationCurve,
        crate::anim_object::text::code::CodeAnimationStyle,
        Option<scal_core::SourceLoc>,
    ),
    CodeRemoveLines(
        Uuid,
        std::ops::Range<u32>,
        Seconds,
        AnimationCurve,
        crate::anim_object::text::code::CodeAnimationStyle,
        Option<scal_core::SourceLoc>,
    ),
    CodeHighlight(Uuid, crate::anim_object::text::code::CodeHighlightAction, Option<scal_core::SourceLoc>),
    Current {
        uuid: Uuid,
        closure: CurrentClosure,
        source_loc: Option<scal_core::SourceLoc>,
    },
    All(Vec<AnimOP>, Option<scal_core::SourceLoc>),
    Sequence(Vec<AnimOP>, Option<scal_core::SourceLoc>),
    Wait(Seconds, Option<scal_core::SourceLoc>),
    PlaySound(Sfx, Seconds, Option<scal_core::SourceLoc>),
}
impl TryInto<Animation> for AnimOP {
    fn try_into(self) -> Result<Animation> {
        // let skip = Box::new(|_, _| Ok(()));
        Ok(match self {
            AnimOP::Instantiate(anim_obj, _loc) => Animation::instant(Box::new(move |animator, _| {
                debug!("Instantiate uuid={}", anim_obj.uuid());
                animator.add_anim_object(anim_obj.clone())?;
                Ok(())
            })),
            AnimOP::TransformMovePos(uuid, pos, duration, curve, _loc) => {
                debug!("TransformMovePos uuid={uuid}");
                transform::move_pos(uuid, pos, duration, curve)
            }
            AnimOP::TransformMoveToObj(moving_uuid, target_uuid, offset, duration, curve, _loc) => {
                debug!("TransformMoveToObj moving={moving_uuid} target={target_uuid}");
                Animation::new(
                    duration,
                    curve,
                    Box::new(move |animator, storage| {
                        let target = animator.get_object_world_matrix(&target_uuid)?;
                        let pos = target.w_axis.xy() + offset;
                        let current = animator.get_object(&moving_uuid)?.transform().position;
                        storage.push(current.x);
                        storage.push(current.y);
                        storage.push(pos.x);
                        storage.push(pos.y);
                        Ok(())
                    }),
                    Some(Box::new(move |animator, t, storage| {
                        let obj = animator.get_object_mut(&moving_uuid)?;
                        let transform = obj.anim_data.transform_mut();
                        transform.position = vec3(
                            storage[0] + t * (storage[2] - storage[0]),
                            storage[1] + t * (storage[3] - storage[1]),
                            transform.position.z,
                        );
                        Ok(())
                    })),
                )
            }
            AnimOP::TransformRotate(uuid, target, duration, curve, _loc) => {
                debug!("TransformRotate uuid={uuid}");
                transform::rotate_to(uuid, target, duration, curve)
            }
            AnimOP::TransformScale(uuid, target, duration, curve, _loc) => {
                debug!("TransformScale uuid={uuid}");
                transform::scale_to(uuid, target, duration, curve)
            }

            AnimOP::CodeAddLines(uuid, text, from_line, duration, curve, style, _loc) => {
                debug!("CodeAddLines uuid={uuid}");
                code::add_lines(uuid, text, from_line, duration, curve, style)
            }
            AnimOP::CodeModifyLine(uuid, line, new_text, duration, curve, style, _loc) => {
                debug!("CodeModifyLine uuid={uuid}");
                code::modify_line(uuid, line, new_text, duration, curve, style)
            }
            AnimOP::CodeRemoveLines(uuid, lines, duration, curve, style, _loc) => {
                debug!("CodeRemoveLines uuid={uuid}");
                code::remove_lines(uuid, lines, duration, curve, style)
            }
            AnimOP::CodeHighlight(uuid, action, _loc) => {
                let (duration, curve) = action.duration_and_curve();
                code::highlight_fade_in(uuid, action, duration, curve)
            }
            AnimOP::All(anim_ops, _loc) => get_all_animation(anim_ops)?,
            AnimOP::Sequence(anim_ops, _loc) => get_sequence_animation(anim_ops)?,
            AnimOP::Current { uuid, closure, source_loc: _ } => {
                Animation::instant(Box::new(move |animator, _| {
                    let mut snapshot = animator.get_object(&uuid)?.anim_data.clone();
                    if let Ok(world) = animator.get_object_world_matrix(&uuid) {
                        let (scale, rot, trans) = world.to_scale_rotation_translation();
                        snapshot.transform_mut().world_uniform = Some(TransformUniform {
                            scale: scale.truncate(),
                            position: trans,
                            rotation: rot.to_euler(glam::EulerRot::ZYX).0.to_degrees(),
                        });
                    }
                    let anim_op = (closure.0)(snapshot);
                    animator.animations_left.push(anim_op);
                    Ok(())
                }))
            }
            AnimOP::Wait(duration, _loc) => Animation::new(
                duration,
                AnimationCurve::Linear,
                Box::new(|_, _| Ok(())),
                Some(Box::new(|_, _, _| Ok(()))),
            ),
            AnimOP::PlaySound(_, _, _) => Animation::instant(Box::new(|_, _| Ok(()))),
        })
    }

    type Error = anyhow::Error;
}
pub fn play(sfx: Sfx, video_delay: Seconds) -> AnimOP {
    AnimOP::PlaySound(sfx, video_delay, None)
}
pub fn sequence(ops: Vec<AnimOP>) -> AnimOP {
    AnimOP::Sequence(ops, None)
}
pub fn all(ops: Vec<AnimOP>) -> AnimOP {
    AnimOP::All(ops, None)
}
pub fn get_sequence_animation(ops: Vec<AnimOP>) -> Result<Animation> {
    let mut child_durations: Vec<f32> = Vec::with_capacity(ops.len());
    let mut total_dur = 0_f32;
    for op in &ops {
        let anim: Animation = op.to_owned().try_into()?;
        let d = if anim.update.is_some() {
            anim.total_duration
        } else {
            0.0
        };
        child_durations.push(d);
        total_dur += d;
    }

    let start = Box::new({
        let loc_ops = ops.to_owned();
        move |animator: &mut Animator, store: &mut Vec<f32>| {
            store.clear();
            for (child_idx, op) in loc_ops.to_owned().into_iter().enumerate() {
                let op_debug = format!("{op:?}");
                let anim: Animation = op.try_into()?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data)
                    .with_context(|| format!("child[{child_idx}] start failed for op {op_debug}"))?;
                store.push(data.len() as f32);
                store.append(&mut data);
            }
            Ok(())
        }
    });

    let update: Option<UpdateAnimationFunction> = if total_dur > 0. {
        let ops_clone = ops.to_owned();
        let durations = child_durations;
        Some(Box::new(
            move |animator: &mut Animator, t: f32, store: &mut Vec<f32>| {
                let abs_time = t * total_dur;
                let mut cumulative = 0_f32;
                let mut store_index = 0;

                for (i, op) in ops_clone.iter().enumerate() {
                    let op_debug = format!("{op:?}");
                    let anim: Animation = op.to_owned().try_into()
                        .with_context(|| format!("failed to convert child[{i}] op {op_debug}"))?;
                    let to_read = store[store_index] as usize;
                    let child_duration = durations[i];

                    if child_duration > 0.
                        && abs_time >= cumulative
                        && abs_time < cumulative + child_duration
                    {
                        let local_t = (abs_time - cumulative) / child_duration;
                        let mut temp_store =
                            store[store_index + 1..store_index + 1 + to_read].to_vec();
                        if let Some(update) = anim.update {
                            (*update)(animator, local_t, &mut temp_store)
                                .with_context(|| format!("child[{i}] update failed for op {op_debug}"))?;
                        }
                        store[store_index + 1..store_index + 1 + to_read]
                            .copy_from_slice(&temp_store);
                        break;
                    }

                    cumulative += child_duration;
                    store_index += to_read + 1;
                }

                Ok(())
            },
        ))
    } else {
        None
    };

    Ok(Animation::new(
        total_dur,
        AnimationCurve::Linear,
        start,
        update,
    ))
}
pub fn get_all_animation(ops: Vec<AnimOP>) -> Result<Animation> {
    let mut dur = 0_f32;
    for operation in &ops {
        let anim: Animation = operation.to_owned().try_into()?;
        if anim.update.is_some() {
            dur = dur.max(anim.total_duration);
        }
    }
    let start = Box::new({
        let loc_ops = ops.to_owned();
        move |animator: &mut Animator, store: &mut Vec<f32>| {
            store.clear();
            for (child_idx, op) in loc_ops.to_owned().into_iter().enumerate() {
                let op_debug = format!("{op:?}");
                let anim: Animation = op.try_into()?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data)
                    .with_context(|| format!("child[{child_idx}] start failed for op {op_debug}"))?;
                store.push(data.len() as f32);
                store.append(&mut data);
            }
            Ok(())
        }
    });
    let update = if dur != 0. {
        let t: Option<UpdateAnimationFunction> = Some(Box::new(
            move |animator: &mut Animator, t: f32, store: &mut Vec<f32>| {
                let mut updated_store = vec![];
                let mut store_index = 0;
                for (child_idx, op) in ops.to_owned().into_iter().enumerate() {
                    let op_debug = format!("{op:?}");
                    let anim: Animation = op.try_into()
                        .with_context(|| format!("failed to convert child[{child_idx}] op {op_debug}"))?;
                    if let Some(update) = anim.update {
                        let to_read = store[store_index] as usize;
                        // +1 to skip the to_read;
                        let mut temp_store =
                            store[store_index + 1..store_index + 1 + to_read].to_vec();
                        let _ = (*update)(animator, t, &mut temp_store);
                        store_index += to_read + 1;
                        updated_store.push(temp_store.len() as f32);
                        updated_store.append(&mut temp_store);
                    }
                }
                *store = updated_store;
                Ok(())
            },
        ));
        t
    } else {
        None
    };

    Ok(Animation::new(dur, AnimationCurve::Linear, start, update))
}

#[derive(Clone, Copy, Debug)]
pub enum AnimationCurve {
    Linear,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInOutBack,
    EaseOutBack,
    EaseInBack,
}
impl AnimationCurve {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            AnimationCurve::Linear => t,

            AnimationCurve::EaseOutCubic => 1.0 - (1.0 - t).powi(3),

            AnimationCurve::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            AnimationCurve::EaseInOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;

                if t < 0.5 {
                    let x = 2.0 * t;
                    (x * x * ((C2 + 1.0) * x - C2)) / 2.0
                } else {
                    let x = 2.0 * t - 2.0;
                    (x * x * ((C2 + 1.0) * x + C2) + 2.0) / 2.0
                }
            }

            AnimationCurve::EaseOutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;

                let x = t - 1.0;
                1.0 + C3 * x * x * x + C1 * x * x
            }

            AnimationCurve::EaseInBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;

                C3 * t * t * t - C1 * t * t
            }
        }
    }
}
type StartAnimationFunction = Box<dyn Fn(&mut Animator, &mut Vec<f32>) -> Result<()>>;
/// Animator + percentage of the animation
type UpdateAnimationFunction = Box<dyn Fn(&mut Animator, f32, &mut Vec<f32>) -> Result<()>>;

pub struct SourceLoc {
    pub file: &'static str,
    pub line: u32,
}

impl std::fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at {}:{}", self.file, self.line)
    }
}

pub struct Animation {
    pub total_duration: f32,
    pub curve: AnimationCurve,
    pub start: StartAnimationFunction,
    pub update: Option<UpdateAnimationFunction>,
    pub location: Option<SourceLoc>,
}
pub fn convert_curve(ease: scal_core::Ease) -> AnimationCurve {
    match ease {
        scal_core::Ease::Linear => AnimationCurve::Linear,
        scal_core::Ease::OutCubic => AnimationCurve::EaseOutCubic,
        scal_core::Ease::InOutCubic => AnimationCurve::EaseInOutCubic,
        scal_core::Ease::InOutBack => AnimationCurve::EaseInOutBack,
        scal_core::Ease::OutBack => AnimationCurve::EaseOutBack,
        scal_core::Ease::InBack => AnimationCurve::EaseInBack,
    }
}
impl Animation {
    #[track_caller]
    pub fn new(
        total_duration: f32,
        curve: AnimationCurve,
        start: StartAnimationFunction,
        update: Option<UpdateAnimationFunction>,
    ) -> Self {
        let loc = std::panic::Location::caller();
        Animation {
            total_duration,
            curve,
            start,
            update,
            location: Some(SourceLoc {
                file: loc.file(),
                line: loc.line(),
            }),
        }
    }
    #[track_caller]
    pub fn instant(start: StartAnimationFunction) -> Self {
        let loc = std::panic::Location::caller();
        Animation {
            total_duration: 0.0,
            curve: AnimationCurve::Linear,
            start,
            update: None,
            location: Some(SourceLoc {
                file: loc.file(),
                line: loc.line(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_curve() {
        let curve = AnimationCurve::Linear;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ease_out_cubic() {
        let curve = AnimationCurve::EaseOutCubic;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((curve.apply(0.5) - 0.875).abs() < f32::EPSILON);
    }

    #[test]
    fn ease_in_out_cubic() {
        let curve = AnimationCurve::EaseInOutCubic;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((curve.apply(0.25) - 0.0625).abs() < f32::EPSILON);
        assert!((curve.apply(0.75) - 0.9375).abs() < f32::EPSILON);
    }

    #[test]
    fn ease_in_out_back() {
        let curve = AnimationCurve::EaseInOutBack;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((curve.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!(
            curve.apply(0.75) > 0.8,
            "expected easeInOutBack to overshoot forward, got {}",
            curve.apply(0.75)
        );
    }

    #[test]
    fn ease_out_back() {
        let curve = AnimationCurve::EaseOutBack;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
        let val = curve.apply(0.5);
        assert!(val > 1.0, "expected easeOutBack to overshoot, got {}", val);
    }

    #[test]
    fn ease_in_back() {
        let curve = AnimationCurve::EaseInBack;
        assert!((curve.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
        let val = curve.apply(0.5);
        assert!(val < 0.5, "expected easeInBack to undershoot, got {}", val);
    }

    #[test]
    fn curve_clamps_input() {
        let curve = AnimationCurve::Linear;
        assert!((curve.apply(-0.5) - 0.0).abs() < f32::EPSILON);
        assert!((curve.apply(1.5) - 1.0).abs() < f32::EPSILON);
    }
}
