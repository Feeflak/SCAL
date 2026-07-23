use anyhow::Context;
use scal_core::{Ease, ScrollOffsetTarget};
use uuid::Uuid;

use crate::anim_object::scroll::ScrollLayout;
use crate::anim_op::{Animation, Seconds};

pub fn scroll_offset_to(
    uuid: Uuid,
    target: ScrollOffsetTarget,
    duration: Seconds,
    curve: Ease,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let obj = animator.get_object(&uuid)?;
            let scroll = obj
                .anim_data
                .as_any()
                .downcast_ref::<ScrollLayout>()
                .context("ScrollOffset animation target is not a ScrollLayout")?;
            let max_scroll = scroll.max_scroll();
            let target_px = match target {
                ScrollOffsetTarget::Percent(p) => p.clamp(0.0, 1.0) * max_scroll,
                ScrollOffsetTarget::Pixels(px) => px.clamp(0.0, max_scroll),
            };
            storage.push(scroll.scroll_offset);
            storage.push(target_px);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let initial = storage[0];
            let target_px = storage[1];
            let new_offset = initial + t * (target_px - initial);

            let obj = animator.get_object_mut(&uuid)?;
            let scroll = obj
                .anim_data
                .as_any_mut()
                .downcast_mut::<ScrollLayout>()
                .context("ScrollOffset animation target is not a ScrollLayout")?;

            scroll.scroll_offset = new_offset;
            animator.apply_scroll_offset(&uuid)?;
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    )
}
