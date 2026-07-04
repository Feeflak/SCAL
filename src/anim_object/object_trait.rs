use std::any::Any;

use uuid::Uuid;

use crate::anim_object::render::PipelineKind;
use crate::anim_object::text::TextManager;
use crate::anim_object::Transform;
use crate::anim_op::AnimOP;
use crate::renderer::{Index, Vertex};

pub type MeshResult = (Vec<Vertex>, Vec<Index>, PipelineKind);

pub trait AnimObjectTrait: Any + std::fmt::Debug + Send {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;
    fn uuid(&self) -> Uuid {
        self.transform().uuid
    }
    fn generate_mesh(&mut self, mgr: &mut TextManager) -> MeshResult;
    fn bind_group_loader(&self) -> Option<BindGroupLoader>;
    fn clone_box(&self) -> Box<dyn AnimObjectTrait>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type BindGroupLoader = Box<dyn Fn(&wgpu::Device, &wgpu::Queue) -> Vec<wgpu::BindGroup>>;

#[derive(Debug)]
pub struct AnimObj(pub Box<dyn AnimObjectTrait>);

impl AnimObj {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::Instantiate(self.clone())
    }
}

impl Clone for AnimObj {
    fn clone(&self) -> Self {
        AnimObj(self.0.clone_box())
    }
}

impl std::ops::Deref for AnimObj {
    type Target = dyn AnimObjectTrait;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::ops::DerefMut for AnimObj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl PartialEq for AnimObj {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}
