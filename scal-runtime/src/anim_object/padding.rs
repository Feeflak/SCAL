use glam::Vec2;

use crate::anim_object::Transform;
use crate::anim_object::object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult};
use crate::anim_object::render::PipelineKind;
use crate::anim_object::text::TextManager;

#[derive(Clone, Debug)]
pub struct Padding {
    pub size: Vec2,
    pub transform: Transform,
}

impl AnimObjectTrait for Padding {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        self.size
    }
    fn generate_mesh(&mut self, _mgr: &mut TextManager) -> MeshResult {
        Ok((vec![], vec![], PipelineKind::Shape))
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
