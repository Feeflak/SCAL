use anyhow::{Context, Result, ensure};
use glam::Vec2;
use scal_core::Ease;
use uuid::Uuid;

use crate::anim_object::vector::Vector;
use crate::anim_op::Animation;
use crate::types::Seconds;

/// The points of the vector, if `uuid` is one.
fn vector_mut<'a>(
    animator: &'a mut crate::animator::Animator,
    uuid: &Uuid,
) -> Result<&'a mut Vector> {
    animator
        .get_object_mut(uuid)?
        .anim_data
        .as_any_mut()
        .downcast_mut::<Vector>()
        .context("VectorMovePoints target is not a vector")
}

/// Moves points of a vector in straight lines to new scene positions.
///
/// Targets are scene coordinates. The points themselves are stored relative to the object, so
/// the position of the object when the animation starts is subtracted.
pub fn move_points(
    uuid: Uuid,
    moves: Vec<(usize, Vec2)>,
    duration: Seconds,
    curve: Ease,
) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let origin = animator.get_object(&uuid)?.transform().position.truncate();
            let vector = vector_mut(animator, &uuid)?;
            // Per move: index, start x, start y, end x, end y.
            for &(index, target) in &moves {
                ensure!(
                    index < vector.points.len(),
                    "vector has {} points, cannot move point {index}",
                    vector.points.len()
                );
                let start = vector.points[index];
                let end = target - origin;
                storage.extend([index as f32, start.x, start.y, end.x, end.y]);
            }
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let vector = vector_mut(animator, &uuid)?;
            for chunk in storage.chunks_exact(5) {
                let start = Vec2::new(chunk[1], chunk[2]);
                let end = Vec2::new(chunk[3], chunk[4]);
                if let Some(point) = vector.points.get_mut(chunk[0] as usize) {
                    *point = start.lerp(end, t);
                }
            }
            animator.regenerate_object_mesh(&uuid)?;
            Ok(())
        })),
    )
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, Vec3, vec2};
    use scal_core::{Camera, Color};

    use crate::anim_object::Transform;
    use crate::anim_object::object_trait::DynAnimObj;
    use crate::anim_op::{AnimOperation, resolve_op};
    use crate::animator::Animator;

    use super::*;

    fn animator_with_vector() -> (Animator, Uuid) {
        let camera = Camera::new(Vec2::new(100.0, 100.0), Vec2::ZERO, 1.0);
        let mut animator =
            Animator::new(vec![AnimOperation::Wait(0.001, None)], 60, camera, 1.0).unwrap();
        let uuid = Uuid::new_v4();
        // Scene points (100, 100) and (200, 100), the object sits in the middle of them.
        animator
            .add_anim_object(DynAnimObj(Box::new(Vector {
                points: vec![vec2(-50.0, 0.0), vec2(50.0, 0.0)],
                color: Color::WHITE,
                thickness: 10.0,
                arrow_head: true,
                head_length: 40.0,
                head_width: 30.0,
                dash: None,
                transform: Transform::with_uuid(uuid, Vec3::new(150.0, 100.0, 0.0)),
            })))
            .unwrap();
        (animator, uuid)
    }

    fn points(animator: &Animator, uuid: &Uuid) -> Vec<Vec2> {
        animator
            .get_object(uuid)
            .unwrap()
            .anim_data
            .as_any()
            .downcast_ref::<Vector>()
            .unwrap()
            .points
            .clone()
    }

    #[test]
    fn moves_only_the_given_points_to_scene_positions() {
        let (mut animator, uuid) = animator_with_vector();
        let anim = resolve_op(AnimOperation::VectorMovePoints(
            uuid,
            vec![(1, vec2(150.0, 300.0))],
            1.0,
            Ease::Linear,
            None,
        ))
        .unwrap();
        let mut store = vec![];
        (anim.start)(&mut animator, &mut store).unwrap();
        let update = anim.update.unwrap();

        update(&mut animator, 0.5, &mut store).unwrap();
        // Halfway between (200, 100) and (150, 300) in the scene, relative to (150, 100).
        assert_eq!(points(&animator, &uuid), [vec2(-50.0, 0.0), vec2(25.0, 100.0)]);

        update(&mut animator, 1.0, &mut store).unwrap();
        assert_eq!(points(&animator, &uuid), [vec2(-50.0, 0.0), vec2(0.0, 200.0)]);
        // The mesh was rebuilt: the arrow head tip is at the new end point.
        let tip = animator.get_object(&uuid).unwrap().render_data.vertices[0].position;
        assert_eq!(tip, vec2(0.0, 200.0));
    }

    #[test]
    fn a_second_move_in_a_sequence_starts_where_the_first_ended() {
        let (mut animator, uuid) = animator_with_vector();
        let move_end = |to: Vec2| {
            AnimOperation::VectorMovePoints(uuid, vec![(1, to)], 1.0, Ease::Linear, None)
        };
        let anim = resolve_op(AnimOperation::Sequence(
            vec![move_end(vec2(200.0, 300.0)), move_end(vec2(200.0, 500.0))],
            None,
        ))
        .unwrap();
        let mut store = vec![];
        (anim.start)(&mut animator, &mut store).unwrap();
        let update = anim.update.unwrap();

        // The point is at scene (200, 100) and goes down 200, then another 200.
        update(&mut animator, 0.5, &mut store).unwrap();
        assert_eq!(points(&animator, &uuid)[1], vec2(50.0, 200.0));
        update(&mut animator, 0.75, &mut store).unwrap();
        assert_eq!(points(&animator, &uuid)[1], vec2(50.0, 300.0));
        update(&mut animator, 1.0, &mut store).unwrap();
        assert_eq!(points(&animator, &uuid)[1], vec2(50.0, 400.0));
    }

    #[test]
    fn out_of_range_point_is_an_error() {
        let (mut animator, uuid) = animator_with_vector();
        let anim = resolve_op(AnimOperation::VectorMovePoints(
            uuid,
            vec![(5, Vec2::ZERO)],
            1.0,
            Ease::Linear,
            None,
        ))
        .unwrap();
        assert!((anim.start)(&mut animator, &mut vec![]).is_err());
    }
}
