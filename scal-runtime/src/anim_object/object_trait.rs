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
    /// The number shown by this object's `{:spec}` placeholders, or `None` if the object
    /// cannot show an animated number. Implementing this and `set_number` is all an object type
    /// needs to work with the `NumberTo` animation.
    fn number(&self) -> Option<f32> {
        None
    }
    /// Shows `number`. Returns `true` if what the object displays changed, so the caller knows
    /// the mesh has to be regenerated.
    fn set_number(&mut self, _number: f32) -> Result<bool> {
        anyhow::bail!("this object cannot show an animated number")
    }
    /// Called once per rendered frame with the seconds since the animation started, for objects
    /// that move by themselves. Returns `true` if the mesh has to be regenerated.
    fn tick(&mut self, _time: f32) -> bool {
        false
    }
    /// Identifies the content a bind group was built from. When it changes the renderer drops
    /// the object's cached bind groups and calls `bind_group_loader` again.
    fn content_version(&self) -> u64 {
        0
    }
    /// Identifies everything the bind groups from `bind_group_loader` are built from, or `None`
    /// if they should not be shared. Objects returning the same key get the same bind groups,
    /// so unchanged images, svgs and equations are not rebuilt (decoded, rasterized,
    /// uploaded) when the animation reloads and every object gets a new uuid.
    fn bind_group_cache_key(&self) -> Option<String> {
        None
    }
    /// Does the CPU work of `bind_group_loader` (decoding, rasterizing) ahead of time. Called
    /// from a background thread when an animation is loaded, so the first frame does not have
    /// to. Only objects with a `bind_group_cache_key` are prewarmed.
    fn prewarm(&self) {}
    fn layout_parent(&self) -> Option<Uuid> {
        self.transform().layout_container
    }
    fn set_layout_parent(&mut self, parent: Option<Uuid>) {
        self.transform_mut().layout_container = parent;
    }
}

/// `path`, with its size and modification time so that an edited file gets a new key.
pub fn file_cache_key(path: &str) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    Some(format!("{path}|{}|{modified}", meta.len()))
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
                pivot: glam::Vec2::ZERO,
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
                pivot: glam::Vec2::ZERO,
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
                pivot: glam::Vec2::ZERO,
            },
        };
        let obj1 = DynAnimObj(Box::new(rect1));
        let obj2 = DynAnimObj(Box::new(rect2));
        let obj3 = DynAnimObj(Box::new(rect3));
        assert_eq!(obj1, obj2);
        assert_ne!(obj1, obj3);
    }
}
