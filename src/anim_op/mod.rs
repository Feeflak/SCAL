pub mod code;
mod transform;

use anyhow::Result;
use glam::Vec2;
use log::debug;
use uuid::Uuid;

use crate::anim_object::object_trait::{AnimObj, AnimObjectTrait};
use crate::animator::Animator;
use crate::types::*;

#[derive(Clone, Debug)]
pub enum AnimOP {
    Instantiate(AnimObj),
    TransformMovePos(Uuid, Vec2, Seconds, AnimationCurve),
    TransformRotate(Uuid, f32, Seconds, AnimationCurve),
    TransformScale(Uuid, Vec2, Seconds, AnimationCurve),
    CodeAddLines(Uuid, String, usize, Seconds, AnimationCurve),
    CodeModifyLine(Uuid, u32, String, Seconds, AnimationCurve),
    CodeRemoveLines(Uuid, std::ops::Range<u32>, Seconds, AnimationCurve),
    All(Vec<AnimOP>),
    Wait(Seconds),
}
impl TryInto<Animation> for AnimOP {
    fn try_into(self) -> Result<Animation> {
        // let skip = Box::new(|_, _| Ok(()));
        Ok(match self {
            AnimOP::Instantiate(anim_obj) => Animation::instant(Box::new(move |animator, _| {
                debug!("Instantiate");
                animator.add_anim_object(anim_obj.clone())?;
                Ok(())
            })),
            AnimOP::TransformMovePos(uuid, pos, duration, curve) => {
                transform::move_pos(uuid, pos, duration, curve)
            }
            AnimOP::TransformRotate(uuid, target, duration, curve) => {
                transform::rotate_to(uuid, target, duration, curve)
            }
            AnimOP::TransformScale(uuid, target, duration, curve) => {
                transform::scale_to(uuid, target, duration, curve)
            }
            AnimOP::CodeAddLines(uuid, text, from_line, duration, curve) => {
                code::add_lines(uuid, text, from_line, duration, curve)
            }
            AnimOP::CodeModifyLine(uuid, line, new_text, duration, curve) => {
                code::modify_line(uuid, line, new_text, duration, curve)
            }
            AnimOP::CodeRemoveLines(uuid, lines, duration, curve) => {
                code::remove_lines(uuid, lines, duration, curve)
            }
            AnimOP::All(anim_ops) => get_all_animation(anim_ops)?,
            AnimOP::Wait(duration) => Animation::new(
                duration,
                AnimationCurve::Linear,
                Box::new(|_, _| Ok(())),
                Some(Box::new(|_, _, _| Ok(()))),
            ),
        })
    }

    type Error = anyhow::Error;
}
pub fn all(ops: Vec<AnimOP>) -> AnimOP {
    AnimOP::All(ops)
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

#[derive(Clone, Debug)]
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
