use glam::{Vec2, vec3};
use uuid::Uuid;

use crate::anim_op::{Animation, AnimationCurve};
use crate::types::*;

pub fn move_pos(uuid: Uuid, target: Vec2, duration: Seconds, curve: AnimationCurve) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let pos = animator.get_object(&uuid)?.transform().position;
            storage.push(pos.x);
            storage.push(pos.y);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let obj = animator.get_object_mut(&uuid)?;
            let transform = obj.anim_data.transform_mut();
            transform.position = vec3(
                storage[0] + t * (target.x - storage[0]),
                storage[1] + t * (target.y - storage[1]),
                transform.position.z,
            );
            Ok(())
        })),
    )
}

pub fn rotate_to(uuid: Uuid, target: f32, duration: Seconds, curve: AnimationCurve) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            storage.push(animator.get_object(&uuid)?.transform().rotation);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let initial = storage[0];
            let obj = animator.get_object_mut(&uuid)?;
            obj.anim_data.transform_mut().rotation = initial + t * (target - initial);
            Ok(())
        })),
    )
}

pub fn scale_to(uuid: Uuid, target: Vec2, duration: Seconds, curve: AnimationCurve) -> Animation {
    Animation::new(
        duration,
        curve,
        Box::new(move |animator, storage| {
            let s = animator.get_object(&uuid)?.transform().scale;
            storage.push(s.x);
            storage.push(s.y);
            Ok(())
        }),
        Some(Box::new(move |animator, t, storage| {
            let obj = animator.get_object_mut(&uuid)?;
            let transform = obj.anim_data.transform_mut();
            transform.scale = Vec2::new(
                storage[0] + t * (target.x - storage[0]),
                storage[1] + t * (target.y - storage[1]),
            );
            Ok(())
        })),
    )
}
