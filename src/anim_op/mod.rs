mod transform;

use anyhow::Result;
use log::debug;
use uuid::Uuid;

use crate::anim_object::AnimObject;
use crate::animator::Animator;
use crate::types::*;

#[derive(Clone, Debug)]
pub enum AnimOP {
    Instantiate(AnimObject),
    TransformMovePos(Uuid, Vec2, Seconds, AnimationCurve),
    TransformAddChildren(Uuid, Vec<AnimObject>),
    All(Vec<AnimOP>),
    Wait(Seconds),
}
impl TryInto<Animation> for AnimOP {
    fn try_into(self) -> Result<Animation> {
        // let skip = Box::new(|_, _| Ok(()));
        Ok(match self {
            AnimOP::Instantiate(anim_object) => Animation::instant(Box::new(move |animator, _| {
                debug!("Instantiate");
                animator.add_anim_object(anim_object.clone());
                Ok(())
            })),
            AnimOP::TransformMovePos(uuid, pos, duration, curve) => Animation::new(
                duration,
                curve,
                Box::new(move |animator, initial_pos_store| {
                    let pos = animator.get_object(&uuid)?.0.transform().pos;
                    initial_pos_store.push(pos.x);
                    initial_pos_store.push(pos.y);
                    Ok(())
                }),
                Some(Box::new(move |animator, t, initial_pos| {
                    // lerp

                    let (base_index, vertices) = {
                        let (anim, render) = animator.get_object(&uuid)?;

                        let transform = anim.transform_mut();
                        let new_pos = Vec2::new(
                            initial_pos[0] + t * (pos.x - initial_pos[0]),
                            initial_pos[1] + t * (pos.y - initial_pos[1]),
                        );
                        transform.pos = new_pos;

                        let vertices = render.transform_updated_vertices(transform);
                        (render.vertices_base_index, vertices)
                    };

                    animator.vertices[base_index..base_index + vertices.len()]
                        .copy_from_slice(&vertices);
                    Ok(())
                })),
            ),
            AnimOP::TransformAddChildren(uuid, anim_objects) => {
                Animation::instant(Box::new(move |animator, _| {
                    for obj in &anim_objects {
                        animator
                            .get_object(&uuid)?
                            .0
                            .transform_mut()
                            .children
                            .push(obj.transform().uuid);
                        animator.add_anim_object(obj.clone());
                    }

                    Ok(())
                }))
            }
            AnimOP::All(anim_ops) => todo!(),
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
