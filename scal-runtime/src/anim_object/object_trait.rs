use std::any::Any;

use anyhow::Result;
use glam::Vec2;
use uuid::Uuid;

use crate::anim_object::render::PipelineKind;
use crate::anim_object::text::TextManager;
use crate::anim_object::{Transform, TransformUniform};
use crate::anim_op::AnimOperation;
use crate::renderer::{Index, Vertex};

pub type MeshResult = Result<(Vec<Vertex>, Vec<Index>, PipelineKind)>;

pub trait AnimObjectTrait: Any + std::fmt::Debug + Send + Sync {
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

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }
    fn layout_parent(&self) -> Option<Uuid> {
        self.transform().layout_container
    }
    fn set_layout_parent(&mut self, parent: Option<Uuid>) {
        self.transform_mut().layout_container = parent;
    }
}

pub type BindGroupLoader = Box<dyn Fn(&wgpu::Device, &wgpu::Queue) -> Vec<wgpu::BindGroup>>;

#[derive(Debug)]
pub struct DynAnimObj(pub Box<dyn AnimObjectTrait>);

impl DynAnimObj {
    pub fn instantiate(&self) -> AnimOperation {
        AnimOperation::Instantiate(self.clone(), None)
    }
    pub fn current_world_uniform(&self) -> Result<TransformUniform> {
        self.transform().get_world_uniform()
    }
}

impl Clone for DynAnimObj {
    fn clone(&self) -> Self {
        DynAnimObj(self.0.clone_box())
    }
}

impl std::ops::Deref for DynAnimObj {
    type Target = dyn AnimObjectTrait;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::ops::DerefMut for DynAnimObj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl PartialEq for DynAnimObj {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim_object::primitive_shapes::Rectangle;
    use glam::{Vec2, Vec3};
    use scal_core::Color;

    #[test]
    fn anim_obj_eq_compares_by_uuid() {
        let id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let rect1 = Rectangle {
            size: Vec2::new(10.0, 10.0),
            corner_radius: 0.0,
            color: Color::WHITE,
            transform: Transform {
                uuid: id,
                parent: None,
                position: Vec3::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
                clip_rect: None,
            },
        };
        let rect2 = Rectangle {
            size: Vec2::new(20.0, 20.0),
            corner_radius: 5.0,
            color: Color::RED,
            transform: Transform {
                uuid: id,
                parent: None,
                position: Vec3::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
                clip_rect: None,
            },
        };
        let rect3 = Rectangle {
            size: Vec2::new(10.0, 10.0),
            corner_radius: 0.0,
            color: Color::WHITE,
            transform: Transform {
                uuid: other_id,
                parent: None,
                position: Vec3::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
                clip_rect: None,
            },
        };
        let obj1 = DynAnimObj(Box::new(rect1));
        let obj2 = DynAnimObj(Box::new(rect2));
        let obj3 = DynAnimObj(Box::new(rect3));
        assert_eq!(obj1, obj2);
        assert_ne!(obj1, obj3);
    }
}
