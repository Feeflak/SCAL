use anyhow::Context;
use scal_core::Ease;
use uuid::Uuid;

use crate::anim_op::Animation;
use crate::types::Seconds;

/// Counts the number an object shows (see `AnimObjectTrait::set_number`) from `from` (or the
/// number it shows when the animation starts) to `target`.
pub fn number_to(
    uuid: Uuid,
    from: Option<f32>,
    target: f32,
    duration: Seconds,
    curve: Ease,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let start = match from {
                Some(from) => from,
                None => animator
                    .get_object(&uuid)?
                    .anim_data
                    .number()
                    .context("NumberTo animation target cannot show a number")?,
            };
            storage.push(start);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let value = storage[0] + t * (target - storage[0]);
            let obj = animator.get_object_mut(&uuid)?;
            if obj.anim_data.set_number(value)? {
                animator.regenerate_object_mesh(&uuid)?;
            }
            Ok(())
        })),
    )
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, Vec3};
    use scal_core::{Camera, Color};

    use crate::anim_object::Transform;
    use crate::anim_object::math::Math;
    use crate::anim_object::object_trait::{AnimObjectTrait, DynAnimObj};
    use crate::anim_object::text::Text;
    use crate::anim_op::AnimOperation;
    use crate::animator::Animator;

    use super::*;

    fn transform() -> Transform {
        Transform::new(None, Vec3::ZERO, 0.0, Vec2::ONE)
    }

    fn text(template: &str, number: f32) -> Text {
        let transform = transform();
        Text {
            id: transform.uuid,
            font_family: "sans-serif".into(),
            align: crate::anim_object::Alignment::Start,
            value: template.into(),
            color: Color::WHITE,
            font_size: 24.0,
            transform,
            cached_size: None,
            modifications: vec![],
            spans: vec![],
            number: 0.0,
            template: None,
        }
        .with_number(number)
    }

    fn run(ops: Vec<AnimOperation>, fps: u32) -> Animator {
        let camera = Camera::new(Vec2::new(200.0, 200.0), Vec2::ZERO, 1.0);
        let mut animator = Animator::new(ops, fps, camera, 1.0).unwrap();
        while animator.animate_next_frame().unwrap().is_some() {}
        animator
    }

    fn shown(animator: &Animator, id: &Uuid) -> String {
        let obj = animator.get_object(id).unwrap();
        if let Some(t) = obj.anim_data.as_any().downcast_ref::<Text>() {
            t.value.clone()
        } else {
            obj.anim_data
                .as_any()
                .downcast_ref::<Math>()
                .unwrap()
                .latex
                .clone()
        }
    }

    // These run through the animator, which lays out glyphs for text. That needs a system
    // font, and the Nix build sandbox has none, so they use math (which embeds its own glyphs).
    fn math_obj(template: &str, number: f32) -> Math {
        Math::new_with_number(
            template.into(),
            Color::WHITE,
            32.0,
            true,
            &[],
            transform(),
            number,
        )
        .unwrap()
    }

    #[test]
    fn counts_from_the_shown_number_to_the_target() {
        let m = math_obj("AoA {:.1}", 2.0);
        assert_eq!(m.latex, "AoA 2.0");
        let id = m.uuid();
        let animator = run(
            vec![
                AnimOperation::Instantiate(DynAnimObj(Box::new(m)), None),
                AnimOperation::NumberTo(id, None, 15.0, 0.5, Ease::Linear, None),
            ],
            30,
        );
        assert_eq!(shown(&animator, &id), "AoA 15.0");
    }

    #[test]
    fn explicit_start_is_used_and_ease_hits_the_end() {
        let m = math_obj("{:.0}", 99.0);
        let id = m.uuid();
        let animator = run(
            vec![
                AnimOperation::Instantiate(DynAnimObj(Box::new(m)), None),
                AnimOperation::NumberTo(id, Some(-10.0), 10.0, 0.5, Ease::InOutCubic, None),
            ],
            30,
        );
        assert_eq!(shown(&animator, &id), "10");
    }

    #[test]
    fn text_updates_without_needing_a_font() {
        let mut t = text("AoA {:.1}", 2.0);
        assert_eq!(t.value, "AoA 2.0");
        assert_eq!(t.number(), Some(2.0));
        assert!(t.set_number(15.0).unwrap());
        assert_eq!(t.value, "AoA 15.0");
        assert!(!t.set_number(15.01).unwrap(), "same text at this precision");
    }

    #[test]
    fn math_counts_and_retypesets() {
        let m = Math::new_with_number(
            r"\alpha = {:.1}^\circ".into(),
            Color::WHITE,
            32.0,
            true,
            &[],
            transform(),
            0.0,
        )
        .unwrap();
        let id = m.uuid();
        let narrow = m.size();
        let animator = run(
            vec![
                AnimOperation::Instantiate(DynAnimObj(Box::new(m)), None),
                AnimOperation::NumberTo(id, Some(0.0), 15.0, 0.5, Ease::Linear, None),
            ],
            30,
        );
        assert_eq!(shown(&animator, &id), r"\alpha = 15.0^\circ");
        let obj = animator.get_object(&id).unwrap();
        assert!(obj.anim_data.size().x > narrow.x, "more digits must be wider");
    }

    #[test]
    fn objects_without_a_placeholder_report_it() {
        let mut t = text("plain", 0.0);
        assert!(t.number().is_none());
        assert!(t.set_number(1.0).is_err());
    }

    #[test]
    fn math_content_version_follows_the_shown_text() {
        let mut m = Math::new_with_number(
            "x = {:.1}".into(),
            Color::WHITE,
            32.0,
            true,
            &[],
            transform(),
            1.0,
        )
        .unwrap();
        let v = m.content_version();
        assert!(!m.set_number(1.04).unwrap(), "same text at this precision");
        assert_eq!(m.content_version(), v);
        assert!(m.set_number(2.0).unwrap());
        assert_ne!(m.content_version(), v);
        assert!(m.set_number(1.0).unwrap());
        assert_eq!(m.content_version(), v, "same content, same version (seek safe)");
    }

    #[test]
    fn spans_share_the_number() {
        let mut t = text("", 0.0);
        t.spans = vec!["a {:.0} ".into(), "b {:.1}".into()];
        t.value = "a {:.0} b {:.1}".into();
        let mut t = t.with_number(3.0);
        assert_eq!(t.value, "a 3 b 3.0");
        assert_eq!(t.spans[1].text, "b 3.0");
        t.set_number(4.0).unwrap();
        assert_eq!(t.spans[0].text, "a 4 ");
    }
}
