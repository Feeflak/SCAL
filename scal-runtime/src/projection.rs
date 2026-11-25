use glam::{Mat4, Quat, Vec2, Vec3};

use crate::anim_object::Transform;

impl Transform {
    /// `T(position + scale * pivot) * R * S * T(-pivot)`.
    ///
    /// The object is scaled around its center, and rotated around `pivot` (an offset from the
    /// center in the object's own unscaled space). With a zero pivot this is the plain
    /// translate-rotate-scale matrix.
    pub fn get_local_matrix(&self) -> Mat4 {
        let anchor = self.scale * self.pivot;
        Mat4::from_translation(Vec3::new(
            self.position.x + anchor.x,
            self.position.y + anchor.y,
            self.position.z,
        )) * Mat4::from_quat(Quat::from_rotation_z(self.rotation.to_radians()))
            * Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0))
            * Mat4::from_translation(Vec3::new(-self.pivot.x, -self.pivot.y, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(pivot: Vec2, rotation: f32, scale: Vec2) -> Transform {
        let mut t = Transform::new(None, Vec3::new(100.0, 50.0, 0.0), rotation, scale);
        t.pivot = pivot;
        t
    }

    #[test]
    fn zero_pivot_matches_plain_trs() {
        let t = t(Vec2::ZERO, 30.0, Vec2::new(2.0, 3.0));
        let plain = Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 3.0, 1.0),
            Quat::from_rotation_z(30f32.to_radians()),
            Vec3::new(100.0, 50.0, 0.0),
        );
        assert!(t.get_local_matrix().abs_diff_eq(plain, 1e-4));
    }

    #[test]
    fn pivot_stays_fixed_while_rotating() {
        let pivot = Vec2::new(-40.0, 5.0);
        let at = |rot| t(pivot, rot, Vec2::ONE).get_local_matrix().transform_point3(pivot.extend(0.0));
        assert!((at(0.0) - at(77.0)).length() < 1e-3);
        assert!((at(0.0) - Vec3::new(60.0, 55.0, 0.0)).length() < 1e-3);
    }

    #[test]
    fn pivot_does_not_move_an_unrotated_object() {
        let p = Vec3::new(10.0, -20.0, 0.0);
        let a = t(Vec2::ZERO, 0.0, Vec2::splat(2.0)).get_local_matrix().transform_point3(p);
        let b = t(Vec2::new(30.0, 7.0), 0.0, Vec2::splat(2.0)).get_local_matrix().transform_point3(p);
        assert!((a - b).length() < 1e-3);
    }
}
