use glam::{Mat4, Quat, Vec2, Vec3, vec3};

use crate::anim_object::Transform;


impl Transform {
    pub fn get_local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation.to_radians()),
            Vec3::new(self.position.x, self.position.y, self.position.z),
        )
    }
}

