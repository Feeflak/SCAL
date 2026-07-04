use std::ops::Range;

use uuid::Uuid;

use crate::anim_object::text::code::Code;
use crate::anim_op::{Animation, AnimationCurve};
use crate::types::*;

fn count_newlines(s: &str) -> usize {
    s.chars().filter(|&c| c == '\n').count()
}

fn code_mut(obj: &mut dyn std::any::Any) -> Option<&mut Code> {
    obj.downcast_mut::<Code>()
}

pub fn add_lines(
    uuid: Uuid,
    text: String,
    from_line: usize,
    duration: Seconds,
    curve: AnimationCurve,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                let mut lines: Vec<&str> = code.source_code.lines().collect();
                let insert_pos = from_line.min(lines.len());
                for (i, line) in text.lines().enumerate() {
                    lines.insert(insert_pos + i, line);
                }
                code.source_code = lines.join("\n");
                code.dirty = true;
            }
            drop(obj);
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        }),
        None,
    )
}

pub fn modify_line(
    uuid: Uuid,
    line: u32,
    new_text: String,
    duration: Seconds,
    curve: AnimationCurve,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                let mut lines: Vec<&str> = code.source_code.lines().collect();
                let idx = (line as usize).min(lines.len().saturating_sub(1));
                lines[idx] = &new_text;
                code.source_code = lines.join("\n");
                code.dirty = true;
            }
            drop(obj);
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        }),
        None,
    )
}

pub fn remove_lines(
    uuid: Uuid,
    lines: Range<u32>,
    duration: Seconds,
    curve: AnimationCurve,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, _storage| {
            let obj = animator.get_object_mut(&uuid)?;
            if let Some(code) = code_mut(obj.anim_data.as_any_mut()) {
                keep_lines(&mut code.source_code, lines.start as usize, lines.end as usize);
                code.dirty = true;
            }
            drop(obj);
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        }),
        None,
    )
}

fn keep_lines(source: &mut String, start: usize, end: usize) {
    let lines: Vec<&str> = source.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i < start || i >= end {
            kept.push(line);
        }
    }
    *source = kept.join("\n");
}
