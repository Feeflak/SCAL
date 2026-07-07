use std::ops::Range;

use log::info;
use uuid::Uuid;

use crate::anim_object::text::code::{Code, CodeAnimationStyle, CodeHighlight, CodeHighlightAction, CodeHighlightKind};
use crate::anim_op::{Animation, AnimationCurve};
use crate::types::*;

fn code_mut(obj: &mut dyn std::any::Any) -> Option<&mut Code> {
    obj.downcast_mut::<Code>()
}

fn insert_lines(source: &mut String, text: &str, from_line: usize) -> usize {
    let text = text.trim_matches('\n');
    let mut lines: Vec<&str> = source.lines().collect();
    let insert_pos = from_line.min(lines.len());
    for (i, line) in text.lines().enumerate() {
        lines.insert(insert_pos + i, line);
    }
    info!("lines: {lines:?}");
    *source = lines.join("\n");
    text.lines().count()
}

fn replace_line(source: &mut String, line: u32, new_text: &str) {
    let mut lines: Vec<&str> = source.lines().collect();
    let idx = (line as usize).min(lines.len().saturating_sub(1));
    lines[idx] = new_text;
    *source = lines.join("\n");
}

fn remove_line_range(source: &mut String, start: usize, end: usize) {
    let lines: Vec<&str> = source.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i < start || i >= end {
            kept.push(line);
        }
    }

    *source = kept.join("\n");
}

pub fn add_lines(
    uuid: Uuid,
    text: String,
    from_line: usize,
    duration: Seconds,
    curve: AnimationCurve,
    style: CodeAnimationStyle,
) -> Animation {
    let new_line_count = text.trim_matches('\n').lines().count();

    let anim_style = style.clone();
    let start = Box::new(
        move |animator: &mut crate::animator::Animator, storage: &mut Vec<f32>| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                // Accumulate line count from any previous completed animation
                let prev_count = code.anim_line_end.saturating_sub(code.anim_line_start);
                code.anim_spacing_accum += prev_count as f32;

                let original_line_count = code.source_code.lines().count();
                let actual_insert_pos = from_line.min(original_line_count);
                insert_lines(&mut code.source_code, &text, from_line);
                code.dirty = true;
                code.anim_reveal = 0.0;
                code.anim_spacing = 0.0;
                code.anim_line_start = actual_insert_pos;
                code.anim_line_end = actual_insert_pos + new_line_count;
                code.anim_style = anim_style.clone();
                storage.push(from_line as f32);
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        },
    );

    let update: Option<
        Box<dyn Fn(&mut crate::animator::Animator, f32, &mut Vec<f32>) -> anyhow::Result<()>>,
    > = match style {
        CodeAnimationStyle::TypeWriter => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                let prev_reveal = code.anim_reveal;
                let prev_spacing = code.anim_spacing;
                if t < 0.3 {
                    code.anim_spacing = t / 0.3;
                    code.anim_reveal = 0.0;
                } else {
                    code.anim_spacing = 1.0;
                    code.anim_reveal = (t - 0.3) / 0.7;
                }
                log::debug!(
                    "TypeWriter add_lines t={:.4} reveal={:.4}->{:.4} spacing={:.4}->{:.4}",
                    t,
                    prev_reveal,
                    code.anim_reveal,
                    prev_spacing,
                    code.anim_spacing
                );
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
        CodeAnimationStyle::Fold => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                let prev_reveal = code.anim_reveal;
                let prev_spacing = code.anim_spacing;
                code.anim_reveal = t;
                code.anim_spacing = t;
                log::debug!(
                    "Fold add_lines t={:.4} reveal={:.4}->{:.4} spacing={:.4}->{:.4}",
                    t,
                    prev_reveal,
                    code.anim_reveal,
                    prev_spacing,
                    code.anim_spacing
                );
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    };

    Animation::new(duration, curve, start, update)
}

pub fn modify_line(
    uuid: Uuid,
    line: u32,
    new_text: String,
    duration: Seconds,
    curve: AnimationCurve,
    style: CodeAnimationStyle,
) -> Animation {
    let anim_style = style.clone();
    let start = Box::new(
        move |animator: &mut crate::animator::Animator, _storage: &mut Vec<f32>| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                replace_line(&mut code.source_code, line, &new_text);
                code.dirty = true;
                code.anim_reveal = 0.0;
                code.anim_spacing = 0.0;
                code.anim_line_start = line as usize;
                code.anim_line_end = line as usize + 1;
                code.anim_style = anim_style.clone();
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        },
    );

    let update: Option<
        Box<dyn Fn(&mut crate::animator::Animator, f32, &mut Vec<f32>) -> anyhow::Result<()>>,
    > = match style {
        CodeAnimationStyle::TypeWriter => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                code.anim_reveal = t;
                code.anim_spacing = 1.0;
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
        CodeAnimationStyle::Fold => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                code.anim_reveal = t;
                code.anim_spacing = t;
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    };

    Animation::new(duration, curve, start, update)
}

pub fn remove_lines(
    uuid: Uuid,
    lines: Range<u32>,
    duration: Seconds,
    curve: AnimationCurve,
    style: CodeAnimationStyle,
) -> Animation {
    let start_line = lines.start;
    let end_line = lines.end;

    let anim_style = style.clone();
    let start = Box::new(
        move |animator: &mut crate::animator::Animator, storage: &mut Vec<f32>| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                remove_line_range(
                    &mut code.source_code,
                    start_line as usize,
                    end_line as usize,
                );
                code.dirty = true;
                code.anim_reveal = 1.0;
                code.anim_spacing = 1.0;
                code.anim_line_start = start_line as usize;
                code.anim_line_end = end_line as usize;
                code.anim_style = anim_style.clone();
                storage.push(start_line as f32);
                storage.push(end_line as f32);
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        },
    );

    let update: Option<
        Box<dyn Fn(&mut crate::animator::Animator, f32, &mut Vec<f32>) -> anyhow::Result<()>>,
    > = match style {
        CodeAnimationStyle::TypeWriter => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                if t < 0.7 {
                    code.anim_reveal = 1.0 - t / 0.7;
                    code.anim_spacing = 1.0;
                } else {
                    code.anim_reveal = 0.0;
                    code.anim_spacing = 1.0 - (t - 0.7) / 0.3;
                }
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
        CodeAnimationStyle::Fold => Some(Box::new(move |animator, t, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                code.anim_reveal = t;
                code.anim_spacing = t;
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    };

    Animation::new(duration, curve, start, update)
}

pub fn highlight_fade_in(
    uuid: Uuid,
    action: CodeHighlightAction,
    duration: Seconds,
    curve: AnimationCurve,
) -> Animation {
    let start = Box::new(
        move |animator: &mut crate::animator::Animator, storage: &mut Vec<f32>| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                let highlight = match &action {
                    CodeHighlightAction::Lines { ranges, color, .. } => CodeHighlight {
                        color: *color,
                        kind: CodeHighlightKind::Lines { ranges: ranges.clone() },
                        progress: 0.0,
                    },
                    CodeHighlightAction::Pattern { regex, color, .. } => CodeHighlight {
                        color: *color,
                        kind: CodeHighlightKind::Pattern { regex: regex.clone() },
                        progress: 0.0,
                    },
                };
                let idx = code.highlights.len();
                code.highlights.push(highlight);
                storage.push(idx as f32);
            }
            let _ = obj;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        },
    );

    let update: Option<
        Box<dyn Fn(&mut crate::animator::Animator, f32, &mut Vec<f32>) -> anyhow::Result<()>>,
    > = Some(Box::new(move |animator, t, storage| {
        let idx = storage.first().copied().unwrap_or(0.0) as usize;
        let obj = animator.get_object_mut(&uuid)?;
        if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
            if let Some(hl) = code.highlights.get_mut(idx) {
                hl.progress = t;
            }
        }
        let _ = obj;
        animator.regenerate_object_mesh(&uuid)?;
        Ok(())
    }));

    Animation::new(duration, curve, start, update)
}
