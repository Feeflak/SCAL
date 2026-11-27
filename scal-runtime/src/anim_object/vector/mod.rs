//! Lines through several points with an optional arrow head, solid or dashed.
use glam::Vec2;
use scal_core::Color;

use crate::anim_object::Transform;
use crate::anim_object::object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult};

pub mod mesh;

#[derive(Clone, Debug)]
pub struct Vector {
    /// Relative to the center of the object.
    pub points: Vec<Vec2>,
    pub color: Color,
    pub thickness: f32,
    pub arrow_head: bool,
    pub head_length: f32,
    pub head_width: f32,
    /// Dash length and gap, `None` for a solid line.
    pub dash: Option<(f32, f32)>,
    pub transform: Transform,
}

impl AnimObjectTrait for Vector {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        let Some(first) = self.points.first() else {
            return Vec2::ZERO;
        };
        let (min, max) = self
            .points
            .iter()
            .fold((*first, *first), |(min, max), p| (min.min(*p), max.max(*p)));
        max - min + Vec2::splat(self.thickness)
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        Ok(mesh::generate_vector_mesh(self))
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
