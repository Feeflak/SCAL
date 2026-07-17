use scal_core::{Ease, TerminalOutputAction};
use uuid::Uuid;

use crate::{
    anim_object::terminal::TerminalTextBuffer,
    anim_op::Animation,
};

/// Animate typing a command into the terminal.
/// Adds a new entry to the terminal and animates char-by-char reveal.
pub fn type_input(
    uuid: Uuid,
    command: String,
    display_override: Option<String>,
    captured_output: String,
    captured_prompt: String,
    duration: f32,
    curve: Ease,
) -> Animation {
    let total_chars = display_override
        .as_deref()
        .unwrap_or(&command)
        .len();

    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let obj = animator.get_object_mut(&uuid)?;
            let buffer = obj
                .anim_data
                .as_any_mut()
                .downcast_mut::<TerminalTextBuffer>()
                .ok_or_else(|| {
                    anyhow::anyhow!("TerminalTypeInput: object {uuid} is not a TerminalTextBuffer")
                })?;

            buffer.add_entry(command.clone(), display_override.clone(), captured_output.clone(), captured_prompt.clone());
            storage.push(total_chars as f32);
            buffer.dirty = true;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let obj = animator.get_object_mut(&uuid)?;
            let buffer = obj
                .anim_data
                .as_any_mut()
                .downcast_mut::<TerminalTextBuffer>()
                .ok_or_else(|| {
                    anyhow::anyhow!("TerminalTypeInput: object {uuid} is not a TerminalTextBuffer")
                })?;

            let total = storage[0] as usize;
            if let Some(entry) = buffer.current_entry_mut() {
                entry.input_reveal = (t * total as f32) as usize;
            }
            buffer.dirty = true;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    )
}

/// Animate terminal output reveal (skip, pull, push, pull_all).
/// Operates on the most recently added entry (current).
pub fn output(
    uuid: Uuid,
    action: TerminalOutputAction,
    duration: f32,
    curve: Ease,
) -> Animation {
    match action {
        TerminalOutputAction::Skip(bytes) => {
            Animation::instant(Box::new(move |animator, _| {
                let obj = animator.get_object_mut(&uuid)?;
                let buffer = obj
                    .anim_data
                    .as_any_mut()
                    .downcast_mut::<TerminalTextBuffer>()
                    .ok_or_else(|| {
                        anyhow::anyhow!("TerminalOutput: object {uuid} is not a TerminalTextBuffer")
                    })?;

                if let Some(entry) = buffer.current_entry_mut() {
                    entry.output_skip = (entry.output_skip + bytes).min(entry.output.len());
                }
                buffer.dirty = true;
                animator.regenerate_object_mesh(&uuid)?;
                Ok(())
            }))
        }
        TerminalOutputAction::Pull(bytes) => {
            Animation::new(
                duration,
                curve,
                Box::new(move |animator, storage| {
                    let obj = animator.get_object_mut(&uuid)?;
                    let buffer = obj
                        .anim_data
                        .as_any_mut()
                        .downcast_mut::<TerminalTextBuffer>()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "TerminalOutput: object {uuid} is not a TerminalTextBuffer"
                            )
                        })?;

                    let current = buffer.current_entry().map_or(0, |e| e.output_reveal);
                    storage.push(current as f32);
                    storage.push(bytes as f32);
                    buffer.dirty = true;
                    animator.regenerate_object_mesh(&uuid)?;
                    Ok(())
                }),
                Some(Box::new(move |animator, t, storage| {
                    let obj = animator.get_object_mut(&uuid)?;
                    let buffer = obj
                        .anim_data
                        .as_any_mut()
                        .downcast_mut::<TerminalTextBuffer>()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "TerminalOutput: object {uuid} is not a TerminalTextBuffer"
                            )
                        })?;

                    if let Some(entry) = buffer.current_entry_mut() {
                        let start = storage[0] as usize;
                        let total_bytes = storage[1] as usize;
                        entry.output_reveal = start + (t * total_bytes as f32) as usize;
                    }
                    buffer.dirty = true;
                    animator.regenerate_object_mesh(&uuid)?;
                    Ok(())
                })),
            )
        }
        TerminalOutputAction::Push(text) => {
            Animation::instant(Box::new(move |animator, _| {
                let obj = animator.get_object_mut(&uuid)?;
                let buffer = obj
                    .anim_data
                    .as_any_mut()
                    .downcast_mut::<TerminalTextBuffer>()
                    .ok_or_else(|| {
                        anyhow::anyhow!("TerminalOutput: object {uuid} is not a TerminalTextBuffer")
                    })?;

                if let Some(entry) = buffer.current_entry_mut() {
                    let existing = entry.pushed_text.take().unwrap_or_default();
                    entry.pushed_text = Some(existing + &text);
                    let total_len = entry.output.len() + entry.pushed_text.as_ref().map_or(0, |s| s.len());
                    entry.output_reveal = total_len.saturating_sub(entry.output_skip);
                }
                buffer.dirty = true;
                animator.regenerate_object_mesh(&uuid)?;
                Ok(())
            }))
        }
        TerminalOutputAction::PullAll => {
            Animation::new(
                duration,
                curve,
                Box::new(move |animator, storage| {
                    let obj = animator.get_object_mut(&uuid)?;
                    let buffer = obj
                        .anim_data
                        .as_any_mut()
                        .downcast_mut::<TerminalTextBuffer>()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "TerminalOutput: object {uuid} is not a TerminalTextBuffer"
                            )
                        })?;

                    if let Some(entry) = buffer.current_entry() {
                        let pushed = entry.pushed_text.as_deref().unwrap_or("");
                        let total = entry.output.len() + pushed.len();
                        let remaining = total.saturating_sub(entry.output_skip);
                        storage.push(entry.output_reveal as f32);
                        storage.push(remaining as f32);
                    } else {
                        storage.push(0.0);
                        storage.push(0.0);
                    }
                    buffer.dirty = true;
                    animator.regenerate_object_mesh(&uuid)?;
                    Ok(())
                }),
                Some(Box::new(move |animator, t, storage| {
                    let obj = animator.get_object_mut(&uuid)?;
                    let buffer = obj
                        .anim_data
                        .as_any_mut()
                        .downcast_mut::<TerminalTextBuffer>()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "TerminalOutput: object {uuid} is not a TerminalTextBuffer"
                            )
                        })?;

                    if let Some(entry) = buffer.current_entry_mut() {
                        let start = storage[0] as usize;
                        let remaining = storage[1] as usize;
                        entry.output_reveal = start + (t * remaining as f32) as usize;
                    }
                    buffer.dirty = true;
                    animator.regenerate_object_mesh(&uuid)?;
                    Ok(())
                })),
            )
        }
    }
}
