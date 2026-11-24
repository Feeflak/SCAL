use anyhow::{Context, Result, anyhow};
use scal_core::{Color, Ease};
use uuid::Uuid;

use crate::anim_object::math::Math;
use crate::anim_op::{Animation, Seconds};

fn part_color(math: &Math, part: &str) -> Result<Color> {
    math.part_colors
        .iter()
        .find(|(n, _)| n == part)
        .map(|(_, c)| *c)
        .ok_or_else(|| {
            let known: Vec<&str> = math.part_colors.iter().map(|(n, _)| n.as_str()).collect();
            anyhow!("math part {part:?} is not defined; known parts: {known:?}")
        })
}

pub fn part_color_to(
    uuid: Uuid,
    part: String,
    target: Color,
    duration: Seconds,
    curve: Ease,
) -> Animation {
    let start_part = part.clone();
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let obj = animator.get_object(&uuid)?;
            let math = obj
                .anim_data
                .as_any()
                .downcast_ref::<Math>()
                .context("MathPartColor animation target is not a Math object")?;
            let c = part_color(math, &start_part)?;
            storage.extend([c.r, c.g, c.b, c.a]);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let mix = |i: usize, to: f32| storage[i] + t * (to - storage[i]);
            let color = Color::new(
                mix(0, target.r),
                mix(1, target.g),
                mix(2, target.b),
                mix(3, target.a),
            );
            let obj = animator.get_object_mut(&uuid)?;
            let math = obj
                .anim_data
                .as_any_mut()
                .downcast_mut::<Math>()
                .context("MathPartColor animation target is not a Math object")?;
            part_color(math, &part)?;
            if let Some((_, c)) = math.part_colors.iter_mut().find(|(n, _)| *n == part) {
                *c = color;
            }
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    )
}

/// Animates the highlight state: highlighted parts tint to `color`, the rest dims to `dim`.
/// An empty `parts` resets (undims and untints). Replacing a highlight crossfades.
pub fn highlight_to(
    uuid: Uuid,
    parts: Vec<String>,
    color: Color,
    dim: f32,
    duration: Seconds,
    curve: Ease,
) -> Animation {
    let start_parts = parts.clone();
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let obj = animator.get_object(&uuid)?;
            let math = obj
                .anim_data
                .as_any()
                .downcast_ref::<Math>()
                .context("MathHighlight animation target is not a Math object")?;
            for p in &start_parts {
                part_color(math, p)?;
            }
            let h = &math.highlight;
            // [active, dim, r, g, b, a, weights...]
            storage.extend([h.active, h.dim, h.color.r, h.color.g, h.color.b, h.color.a]);
            storage.extend(h.weights.iter().copied());
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let obj = animator.get_object_mut(&uuid)?;
            let math = obj
                .anim_data
                .as_any_mut()
                .downcast_mut::<Math>()
                .context("MathHighlight animation target is not a Math object")?;
            let mix = |i: usize, to: f32| storage[i] + t * (to - storage[i]);
            let on = !parts.is_empty();
            // A reset keeps the current dim and color so only the weights fade out.
            let (dim_to, color_to) = if on {
                (dim, color)
            } else {
                (storage[1], Color::new(storage[2], storage[3], storage[4], storage[5]))
            };
            let targets: Vec<f32> = math
                .part_colors
                .iter()
                .map(|(n, _)| if parts.contains(n) { 1.0 } else { 0.0 })
                .collect();
            let h = &mut math.highlight;
            h.active = mix(0, if on { 1.0 } else { 0.0 });
            h.dim = mix(1, dim_to);
            h.color = Color::new(
                mix(2, color_to.r),
                mix(3, color_to.g),
                mix(4, color_to.b),
                mix(5, color_to.a),
            );
            for (i, w) in h.weights.iter_mut().enumerate() {
                *w = mix(6 + i, targets[i]);
            }
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    )
}
