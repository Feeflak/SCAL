use glam::{Vec2, vec2};
use scal_core::prelude::*;

const POP: Motion = Motion::new(0.5, Ease::OutBack);

fn is_op<T: PartialEq + std::fmt::Debug>(got: T, want: T) {
    assert_eq!(got, want);
}

#[test]
fn with_sets_duration_and_ease() {
    let r = rectangle().build();
    match r.scale_to(2.0).with(POP).into_anim_op() {
        AnimOP::TransformScale(_, target, dur, ease, _) => {
            is_op(target, Vec2::splat(2.0));
            is_op(dur, 0.5);
            is_op(ease, Ease::OutBack);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn shortcuts_match_the_long_form() {
    let r = rectangle().build();
    match r.move_to(vec2(1.0, 2.0)).into_anim_op() {
        AnimOP::TransformMovePos(id, p, ..) => {
            is_op(id, r.transform.uuid);
            is_op(p, vec2(1.0, 2.0));
        }
        other => panic!("{other:?}"),
    }
    match r.rotate_to(45.0).into_anim_op() {
        AnimOP::TransformRotate(_, deg, ..) => is_op(deg, 45.0),
        other => panic!("{other:?}"),
    }
    match r.scale_to(vec2(1.0, 3.0)).into_anim_op() {
        AnimOP::TransformScale(_, s, ..) => is_op(s, vec2(1.0, 3.0)),
        other => panic!("{other:?}"),
    }
}

#[test]
fn pop_in_and_out() {
    let r = rectangle().hidden().build();
    is_op(r.transform.scale, Vec2::ZERO);
    match r.pop_in().into_anim_op() {
        AnimOP::TransformScale(_, s, d, e, _) => {
            is_op((s, d, e), (Vec2::ONE, 0.5, Ease::OutBack));
        }
        other => panic!("{other:?}"),
    }
    match r.pop_out().over(0.1).into_anim_op() {
        AnimOP::TransformScale(_, s, d, e, _) => {
            is_op((s, d, e), (Vec2::ZERO, 0.1, Ease::InBack));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn builder_scale_takes_f32_or_vec2() {
    is_op(
        rectangle().scale(0.5).build().transform.scale,
        Vec2::splat(0.5),
    );
    is_op(
        rectangle().scale(Vec2::ONE * 3.0).build().transform.scale,
        Vec2::splat(3.0),
    );
}

#[test]
fn group_animates_all_in_parallel() {
    let (a, b) = (rectangle().build(), circle().build());
    let g = group![a, b];
    match g.scale_to(0.0).with(POP).into_anim_op() {
        AnimOP::All(ops, _) => {
            assert_eq!(ops.len(), 2);
            for (op, id) in ops.iter().zip([a.transform.uuid, b.transform.uuid]) {
                match op {
                    AnimOP::TransformScale(i, s, d, e, _) => {
                        is_op((*i, *s, *d, *e), (id, Vec2::ZERO, 0.5, Ease::OutBack));
                    }
                    other => panic!("{other:?}"),
                }
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn spawn_instantiates_everything() {
    let (a, b) = (rectangle().build(), circle().build());
    match spawn![a, b] {
        AnimOP::All(ops, _) => {
            assert_eq!(ops.len(), 2);
            assert!(ops.iter().all(|o| matches!(o, AnimOP::Instantiate(..))));
        }
        other => panic!("{other:?}"),
    }
    // usable as a timeline item
    assert_eq!(timeline![spawn![a, b], wait(1.0)].len(), 2);
}

fn palette(h: &MathHandle) -> Vec<String> {
    match &h.kind {
        AnimObjKind::Math { palette, .. } => palette.iter().map(|(n, _)| n.clone()).collect(),
        _ => unreachable!(),
    }
}

use scal_core::anim_obj::AnimObjKind;

#[test]
fn math_style_applies_and_skips_unused_vars() {
    let red = color(1.0, 0.0, 0.0, 1.0);
    let style = math_style()
        .font_size(100.0)
        .vars([("a", red), ("b", Color::WHITE)])
        .highlight(Color::GREEN, 0.1);
    let eq = math(r"\textcolor{a}{a} + 1").style(&style).build();
    assert_eq!(palette(&eq), ["a"]);
    match &eq.kind {
        AnimObjKind::Math { font_size, .. } => is_op(*font_size, 100.0),
        _ => unreachable!(),
    }
    // highlight defaults come from the style, and can still be overridden
    match eq.highlight(["a"]).into_anim_op() {
        AnimOP::MathHighlight(_, parts, c, dim, ..) => {
            is_op((parts, c, dim), (vec!["a".to_string()], Color::GREEN, 0.1));
        }
        other => panic!("{other:?}"),
    }
    match eq.highlight(["a"]).dim(0.5).into_anim_op() {
        AnimOP::MathHighlight(_, _, _, dim, ..) => is_op(dim, 0.5),
        other => panic!("{other:?}"),
    }
}

#[test]
fn builder_calls_after_style_win_and_var_color_replaces() {
    let style = math_style().font_size(100.0).var("a", Color::RED);
    let eq = math(r"\textcolor{a}{a}")
        .style(&style)
        .font_size(10.0)
        .var_color("a", Color::BLUE)
        .build();
    assert_eq!(palette(&eq), ["a"]);
    match &eq.kind {
        AnimObjKind::Math {
            font_size, palette, ..
        } => {
            is_op(*font_size, 10.0);
            is_op(palette[0].1, Color::BLUE);
        }
        _ => unreachable!(),
    }
}

#[test]
fn default_highlight_is_unchanged() {
    let eq = math(r"\textcolor{a}{a}").var_color("a", Color::RED).build();
    match eq.highlight(["a"]).into_anim_op() {
        AnimOP::MathHighlight(_, _, c, dim, ..) => {
            is_op((c, dim), (color(1.0, 0.9, 0.25, 1.0), 0.25))
        }
        other => panic!("{other:?}"),
    }
}
