pub mod code;
mod transform;

use anyhow::Result;
use glam::{Vec2, Vec4Swizzles, vec3};
use log::debug;
use uuid::Uuid;

use crate::anim_object::object_trait::AnimObj;
use crate::anim_object::TransformUniform;
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
    Instantiate(AnimObj),
    TransformMovePos(Uuid, Vec2, Seconds, AnimationCurve),
    TransformMoveToObj(Uuid, Uuid, Vec2, Seconds, AnimationCurve),
    TransformRotate(Uuid, f32, Seconds, AnimationCurve),
    TransformScale(Uuid, Vec2, Seconds, AnimationCurve),

    CodeAddLines(Uuid, String, usize, Seconds, AnimationCurve, crate::anim_object::text::code::CodeAnimationStyle),
    CodeModifyLine(Uuid, u32, String, Seconds, AnimationCurve, crate::anim_object::text::code::CodeAnimationStyle),
    CodeRemoveLines(Uuid, std::ops::Range<u32>, Seconds, AnimationCurve, crate::anim_object::text::code::CodeAnimationStyle),
    CodeHighlight(Uuid, crate::anim_object::text::code::CodeHighlightAction),
    Current { uuid: Uuid, closure: CurrentClosure },
    All(Vec<AnimOP>),
    Sequence(Vec<AnimOP>),
    Wait(Seconds),
    PlaySound(Sfx, Seconds),
}
impl TryInto<Animation> for AnimOP {
    fn try_into(self) -> Result<Animation> {
        // let skip = Box::new(|_, _| Ok(()));
        Ok(match self {
            AnimOP::Instantiate(anim_obj) => Animation::instant(Box::new(move |animator, _| {
                debug!("Instantiate uuid={}", anim_obj.uuid());
                animator.add_anim_object(anim_obj.clone())?;
                Ok(())
            })),
            AnimOP::TransformMovePos(uuid, pos, duration, curve) => {
                debug!("TransformMovePos uuid={uuid}");
                transform::move_pos(uuid, pos, duration, curve)
            }
            AnimOP::TransformMoveToObj(moving_uuid, target_uuid, offset, duration, curve) => {
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
            AnimOP::TransformRotate(uuid, target, duration, curve) => {
                debug!("TransformRotate uuid={uuid}");
                transform::rotate_to(uuid, target, duration, curve)
            }
            AnimOP::TransformScale(uuid, target, duration, curve) => {
                debug!("TransformScale uuid={uuid}");
                transform::scale_to(uuid, target, duration, curve)
            }

            AnimOP::CodeAddLines(uuid, text, from_line, duration, curve, style) => {
                debug!("CodeAddLines uuid={uuid}");
                code::add_lines(uuid, text, from_line, duration, curve, style)
            }
            AnimOP::CodeModifyLine(uuid, line, new_text, duration, curve, style) => {
                debug!("CodeModifyLine uuid={uuid}");
                code::modify_line(uuid, line, new_text, duration, curve, style)
            }
            AnimOP::CodeRemoveLines(uuid, lines, duration, curve, style) => {
                debug!("CodeRemoveLines uuid={uuid}");
                code::remove_lines(uuid, lines, duration, curve, style)
            }
            AnimOP::CodeHighlight(uuid, action) => {
                let (duration, curve) = action.duration_and_curve();
                code::highlight_fade_in(uuid, action, duration, curve)
            }
            AnimOP::All(anim_ops) => get_all_animation(anim_ops)?,
            AnimOP::Sequence(anim_ops) => get_sequence_animation(anim_ops)?,
            AnimOP::Current { uuid, closure } => Animation::instant(Box::new(move |animator, _| {
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
            })),
            AnimOP::Wait(duration) => Animation::new(
                duration,
                AnimationCurve::Linear,
                Box::new(|_, _| Ok(())),
                Some(Box::new(|_, _, _| Ok(()))),
            ),
            AnimOP::PlaySound(_, _) => Animation::instant(Box::new(|_, _| Ok(()))),
        })
    }

    type Error = anyhow::Error;
}
pub fn play(sfx: Sfx, video_delay: Seconds) -> AnimOP {
    AnimOP::PlaySound(sfx, video_delay)
}
pub fn sequence(ops: Vec<AnimOP>) -> AnimOP {
    AnimOP::Sequence(ops)
}
pub fn all(ops: Vec<AnimOP>) -> AnimOP {
    AnimOP::All(ops)
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
            for op in loc_ops.to_owned() {
                let anim: Animation = op.try_into()?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data)?;
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
                    let anim: Animation = op.to_owned().try_into()?;
                    let to_read = store[store_index] as usize;
                    let child_duration = durations[i];

                    if child_duration > 0. && abs_time >= cumulative
                        && abs_time < cumulative + child_duration
                    {
                        let local_t = (abs_time - cumulative) / child_duration;
                        let mut temp_store =
                            store[store_index + 1..store_index + 1 + to_read].to_vec();
                        if let Some(update) = anim.update {
                            (*update)(animator, local_t, &mut temp_store)?;
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

    Ok(Animation::new(total_dur, AnimationCurve::Linear, start, update))
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
            for op in loc_ops.to_owned() {
                let anim: Animation = op.try_into()?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data)?;
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
                for op in ops.to_owned() {
                    let anim: Animation = op.try_into()?;
                    if let Some(update) = anim.update {
                        let to_read = store[store_index] as usize;
                        // +1 to skip the to_read;
                        let mut temp_store =
                            store[store_index + 1..store_index + 1 + to_read].to_vec();
                        (*update)(animator, t, &mut temp_store);
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

pub struct Animation {
    pub total_duration: f32,
    pub curve: AnimationCurve,
    pub start: StartAnimationFunction,
    pub update: Option<UpdateAnimationFunction>,
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
    pub fn new(
        total_duration: f32,
        curve: AnimationCurve,
        start: StartAnimationFunction,
        update: Option<UpdateAnimationFunction>,
    ) -> Self {
        Animation {
            total_duration,
            curve,
            start,
            update,
        }
    }
    pub fn instant(start: StartAnimationFunction) -> Self {
        Animation {
            total_duration: 0.0,
            curve: AnimationCurve::Linear,
            start,
            update: None,
        }
    }
}
