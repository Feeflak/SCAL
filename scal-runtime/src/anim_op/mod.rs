pub mod code;
mod scroll;
pub mod terminal;
mod transform;

use std::any::Any;

use crate::types::Seconds;

use std::ops::Range;

use anyhow::{Context, Result};
use glam::{Vec2, Vec4Swizzles, vec3};
use log::debug;
use scal_core::{
    CodeAnimationStyle, CodeHighlightAction, Color, Ease, ScrollOffsetTarget, TerminalOutputAction,
    Time, Sfx, SourceLoc,
};
use uuid::Uuid;

use crate::anim_object::object_trait::DynAnimObj;
use crate::animator::Animator;

#[derive(Clone, Debug)]
pub enum AnimOperation {
    Instantiate(DynAnimObj, Option<SourceLoc>),
    TransformMovePos(Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    TransformMoveToObj(Uuid, Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    TransformRotate(Uuid, f32, Time, Ease, Option<SourceLoc>),
    TransformScale(Uuid, Vec2, Time, Ease, Option<SourceLoc>),
    CodeAddLines(
        Uuid,
        String,
        usize,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeModifyLine(
        Uuid,
        u32,
        String,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeRemoveLines(
        Uuid,
        Range<u32>,
        Time,
        Ease,
        CodeAnimationStyle,
        Option<SourceLoc>,
    ),
    CodeHighlight(Uuid, CodeHighlightAction, Option<SourceLoc>),
    TerminalTypeInput(Uuid, String, Option<String>, String, String, Time, Ease, Option<CodeAnimationStyle>, Option<SourceLoc>),
    TerminalOutput(Uuid, TerminalOutputAction, Time, Ease, Option<CodeAnimationStyle>, Option<SourceLoc>),
    ObjectColor(Uuid, Color, Time, Ease, Option<SourceLoc>),
    ScrollOffset(Uuid, ScrollOffsetTarget, Time, Ease, Option<SourceLoc>),
    All(Vec<Self>, Option<SourceLoc>),
    Sequence(Vec<Self>, Option<SourceLoc>),
    Wait(Time, Option<SourceLoc>),
    PlaySound(Sfx, Time, Option<SourceLoc>),
}

impl AnimOperation {
    pub fn location(&self) -> Option<&SourceLoc> {
        match self {
            AnimOperation::Instantiate(_, l)
            | AnimOperation::TransformMovePos(_, _, _, _, l)
            | AnimOperation::TransformMoveToObj(_, _, _, _, _, l)
            | AnimOperation::TransformRotate(_, _, _, _, l)
            | AnimOperation::TransformScale(_, _, _, _, l)
            | AnimOperation::CodeAddLines(_, _, _, _, _, _, l)
            | AnimOperation::CodeModifyLine(_, _, _, _, _, _, l)
            | AnimOperation::CodeRemoveLines(_, _, _, _, _, l)
            | AnimOperation::CodeHighlight(_, _, l)
            | AnimOperation::TerminalTypeInput(_, _, _, _, _, _, _, _, l)
            | AnimOperation::TerminalOutput(_, _, _, _, _, l)
            | AnimOperation::ObjectColor(_, _, _, _, l)
            | AnimOperation::ScrollOffset(_, _, _, _, l)
            | AnimOperation::All(_, l)
            | AnimOperation::Sequence(_, l)
            | AnimOperation::Wait(_, l)
            | AnimOperation::PlaySound(_, _, l) => l.as_ref(),
        }
    }
}

fn ensure_duration(d: Time, label: &str) -> Result<()> {
    anyhow::ensure!(d > 0.0, "{label} duration must be > 0, got {d}");
    Ok(())
}

pub fn resolve_op(op: AnimOperation) -> Result<Animation> {
    Ok(match op {
        AnimOperation::Instantiate(anim_obj, _loc) => {
            Animation::instant(Box::new(move |animator, _| {
                debug!("Instantiate uuid={}", anim_obj.uuid());
                animator.add_anim_object(anim_obj.clone())?;
                Ok(())
            }))
        }
        AnimOperation::TransformMovePos(uuid, pos, duration, curve, _loc) => {
            debug!("TransformMovePos uuid={uuid}");
            ensure_duration(duration, "TransformMovePos")?;
            transform::move_pos(uuid, pos, duration, curve)
        }
        AnimOperation::TransformMoveToObj(
            moving_uuid,
            target_uuid,
            offset,
            duration,
            curve,
            _loc,
        ) => {
            debug!("TransformMoveToObj moving={moving_uuid} target={target_uuid}");
            ensure_duration(duration, "TransformMoveToObj")?;
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
        AnimOperation::TransformRotate(uuid, target, duration, curve, _loc) => {
            debug!("TransformRotate uuid={uuid}");
            ensure_duration(duration, "TransformRotate")?;
            transform::rotate_to(uuid, target, duration, curve)
        }
        AnimOperation::TransformScale(uuid, target, duration, curve, _loc) => {
            debug!("TransformScale uuid={uuid}");
            ensure_duration(duration, "TransformScale")?;
            transform::scale_to(uuid, target, duration, curve)
        }
        AnimOperation::ObjectColor(uuid, target, duration, curve, _loc) => {
            debug!("ObjectColor uuid={uuid}");
            ensure_duration(duration, "ObjectColor")?;
            color_to(uuid, target, duration, curve)
        }
        AnimOperation::ScrollOffset(uuid, target, duration, curve, _loc) => {
            debug!("ScrollOffset uuid={uuid} target={target:?}");
            ensure_duration(duration, "ScrollOffset")?;
            scroll::scroll_offset_to(uuid, target, duration, curve)
        }

        AnimOperation::CodeAddLines(uuid, text, from_line, duration, curve, style, _loc) => {
            debug!("CodeAddLines uuid={uuid}");
            ensure_duration(duration, "CodeAddLines")?;
            code::add_lines(uuid, text, from_line, duration, curve, style)
        }
        AnimOperation::CodeModifyLine(uuid, line, new_text, duration, curve, style, _loc) => {
            debug!("CodeModifyLine uuid={uuid}");
            ensure_duration(duration, "CodeModifyLine")?;
            code::modify_line(uuid, line, new_text, duration, curve, style)
        }
        AnimOperation::CodeRemoveLines(uuid, lines, duration, curve, style, _loc) => {
            debug!("CodeRemoveLines uuid={uuid}");
            ensure_duration(duration, "CodeRemoveLines")?;
            code::remove_lines(uuid, lines, duration, curve, style)
        }
        AnimOperation::CodeHighlight(uuid, action, _loc) => {
            let (duration, curve) = action.duration_and_curve();
            ensure_duration(duration, "CodeHighlight")?;
            code::highlight_fade_in(uuid, action, duration, curve)
        }
        AnimOperation::TerminalTypeInput(uuid, command, display_override, captured_output, captured_prompt, duration, curve, style, _loc) => {
            debug!("TerminalTypeInput uuid={uuid}");
            ensure_duration(duration, "TerminalTypeInput")?;
            self::terminal::type_input(uuid, command, display_override, captured_output, captured_prompt, duration, curve, style)
        }
        AnimOperation::TerminalOutput(uuid, action, duration, curve, style, _loc) => {
            debug!("TerminalOutput uuid={uuid} action={action:?}");
            match action {
                TerminalOutputAction::Pull(_) | TerminalOutputAction::PullAll | TerminalOutputAction::PullLine => {
                    ensure_duration(duration, "TerminalOutput")?;
                }
                _ => {}
            }
            self::terminal::output(uuid, action, duration, curve, style)
        }
        AnimOperation::All(anim_ops, _loc) => get_all_animation(anim_ops)?,
        AnimOperation::Sequence(anim_ops, _loc) => get_sequence_animation(anim_ops)?,
        // AnimOP::Current {
        //     uuid,
        //     closure,
        //     source_loc: _,
        // } => Animation::instant(Box::new(move |animator, _| {
        //     let mut snapshot = animator.get_object(&uuid)?.anim_data.clone();
        //     if let Ok(world) = animator.get_object_world_matrix(&uuid) {
        //         let (scale, rot, trans) = world.to_scale_rotation_translation();
        //         snapshot.transform_mut().world_uniform = Some(TransformUniform {
        //             scale: scale.truncate(),
        //             position: trans,
        //             rotation: rot.to_euler(glam::EulerRot::ZYX).0.to_degrees(),
        //         });
        //     }
        //     let anim_op = (closure.0)(snapshot);
        //     animator.animations_left.push(anim_op);
        //     Ok(())
        // })),
        AnimOperation::Wait(duration, _loc) => {
            ensure_duration(duration, "Wait")?;
            Animation::new(
                duration,
                Ease::Linear,
                Box::new(|_, _| Ok(())),
                Some(Box::new(|_, _, _| Ok(()))),
            )
        }
        AnimOperation::PlaySound(_, _, _) => Animation::instant(Box::new(|_, _| Ok(()))),
    })
}

pub fn play(sfx: Sfx, video_delay: Time) -> AnimOperation {
    AnimOperation::PlaySound(sfx, video_delay, None)
}
pub fn sequence(ops: Vec<AnimOperation>) -> AnimOperation {
    AnimOperation::Sequence(ops, None)
}
pub fn all(ops: Vec<AnimOperation>) -> AnimOperation {
    AnimOperation::All(ops, None)
}
pub fn get_sequence_animation(ops: Vec<AnimOperation>) -> Result<Animation> {
    let mut child_durations: Vec<f32> = Vec::with_capacity(ops.len());
    let mut total_dur = 0_f32;
    for op in &ops {
        let anim: Animation = resolve_op(op.to_owned())?;
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

                let anim: Animation = resolve_op(op.to_owned())?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data).with_context(|| {
                    format!("child[{child_idx}] start failed for op {op_debug}")
                })?;
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

                    let anim: Animation = resolve_op(op.to_owned())
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
                            (*update)(animator, local_t, &mut temp_store).with_context(|| {
                                format!("child[{i}] update failed for op {op_debug}")
                            })?;
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

    Ok(Animation::new(total_dur, Ease::Linear, start, update))
}
pub fn get_all_animation(ops: Vec<AnimOperation>) -> Result<Animation> {
    let mut dur = 0_f32;
    for operation in &ops {
        let anim: Animation = resolve_op(operation.to_owned())?;
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
                let anim: Animation = resolve_op(op.to_owned())?;
                let mut data = vec![];
                (*anim.start)(animator, &mut data).with_context(|| {
                    format!("child[{child_idx}] start failed for op {op_debug}")
                })?;
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

                    let anim: Animation = resolve_op(op.to_owned()).with_context(|| {
                        format!("failed to convert child[{child_idx}] op {op_debug}")
                    })?;
                    let to_read = store[store_index] as usize;
                    // +1 to skip the to_read;
                    let mut temp_store =
                        store[store_index + 1..store_index + 1 + to_read].to_vec();
                    if let Some(update) = anim.update {
                        (*update)(animator, t, &mut temp_store).with_context(|| {
                            format!("child[{child_idx}] update failed for op {op_debug}")
                        })?;
                    }
                    store_index += to_read + 1;
                    updated_store.push(temp_store.len() as f32);
                    updated_store.append(&mut temp_store);
                }
                *store = updated_store;
                Ok(())
            },
        ));
        t
    } else {
        None
    };

    Ok(Animation::new(dur, Ease::Linear, start, update))
}

fn set_object_color(obj: &mut dyn Any, color: Color) {
    if let Some(rect) = obj.downcast_mut::<crate::anim_object::primitive_shapes::Rectangle>() {
        rect.color = color;
    } else if let Some(circle) = obj.downcast_mut::<crate::anim_object::primitive_shapes::Circle>() {
        circle.color = color;
    } else if let Some(poly) = obj.downcast_mut::<crate::anim_object::primitive_shapes::Polygon>() {
        poly.color = color;
    } else if let Some(text) = obj.downcast_mut::<crate::anim_object::text::Text>() {
        text.color = color;
    } else if let Some(image) = obj.downcast_mut::<crate::anim_object::image::Image>() {
        image.color = color;
    }
}

fn get_object_color(obj: &dyn Any) -> Option<Color> {
    if let Some(rect) = obj.downcast_ref::<crate::anim_object::primitive_shapes::Rectangle>() {
        Some(rect.color)
    } else if let Some(circle) = obj.downcast_ref::<crate::anim_object::primitive_shapes::Circle>() {
        Some(circle.color)
    } else if let Some(poly) = obj.downcast_ref::<crate::anim_object::primitive_shapes::Polygon>() {
        Some(poly.color)
    } else if let Some(text) = obj.downcast_ref::<crate::anim_object::text::Text>() {
        Some(text.color)
    } else if let Some(image) = obj.downcast_ref::<crate::anim_object::image::Image>() {
        Some(image.color)
    } else {
        None
    }
}

fn color_to(uuid: Uuid, target: Color, duration: Seconds, curve: Ease) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let obj = animator.get_object(&uuid)?;
            if let Some(initial) = get_object_color(obj.anim_data.as_any()) {
                storage.push(initial.r);
                storage.push(initial.g);
                storage.push(initial.b);
                storage.push(initial.a);
            }
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            if storage.len() < 4 {
                return Ok(());
            }
            let r = storage[0] + t * (target.r - storage[0]);
            let g = storage[1] + t * (target.g - storage[1]);
            let b = storage[2] + t * (target.b - storage[2]);
            let a = storage[3] + t * (target.a - storage[3]);
            let obj = animator.get_object_mut(&uuid)?;
            set_object_color(obj.anim_data.as_any_mut(), Color::new(r, g, b, a));
            Ok(())
        })),
    )
}

type StartAnimationFunction = Box<dyn Fn(&mut Animator, &mut Vec<f32>) -> Result<()>>;
/// Animator + percentage of the animation
type UpdateAnimationFunction = Box<dyn Fn(&mut Animator, f32, &mut Vec<f32>) -> Result<()>>;

pub struct Animation {
    pub total_duration: f32,
    pub curve: Ease,
    pub start: StartAnimationFunction,
    pub update: Option<UpdateAnimationFunction>,
    pub location: Option<SourceLoc>,
}
impl Animation {
    #[track_caller]
    pub fn new(
        total_duration: f32,
        curve: Ease,
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
                file: loc.file().to_string(),
                line: loc.line(),
            }),
        }
    }
    #[track_caller]
    pub fn instant(start: StartAnimationFunction) -> Self {
        let loc = std::panic::Location::caller();
        Animation {
            total_duration: 0.0,
            curve: Ease::Linear,
            start,
            update: None,
            location: Some(SourceLoc {
                file: loc.file().to_string(),
                line: loc.line(),
            }),
        }
    }
}
