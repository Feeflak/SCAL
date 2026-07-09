use std::collections::HashMap;

use crate::{
    anim_object::{
        Transform,
        compose::LayoutContainer,
        object_trait::AnimObj,
        primitive_shapes::Rectangle,
        render::{ObjectRenderData, PipelineKind},
        text::{TextManager, atlas::GlyphUpdateData},
    },
    anim_op::{AnimOP, Animation},
    anim_render::AnimationState,
    projection::Camera,
    renderer::{Index, Vertex},
    types::Color,
};
use anyhow::{Context, Result, bail};
use glam::Mat4;
use log::debug;
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct Object {
    pub anim_data: AnimObj,
    pub render_data: ObjectRenderData,
}
impl Object {
    pub fn transform(&self) -> &Transform {
        self.anim_data.transform()
    }
    pub fn uuid(&self) -> &Uuid {
        &self.anim_data.transform().uuid
    }
    pub fn z_pos(&self) -> f32 {
        self.render_data.world_matrix_cache.w_axis.z
    }
}

pub struct Animator {
    pub camera: Camera,
    pub fps: u32,
    pub anim_state: AnimationState,
    pub animations_left: Vec<AnimOP>,

    pub objects_lookup: HashMap<Uuid, usize>,
    pub objects: Vec<Object>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub text_manager: TextManager,
    pub layout_containers: HashMap<Uuid, LayoutContainer>,
    pub layout_resizing_in_progress: bool,
}

pub struct Scene<'a> {
    pub camera: &'a Camera,
    pub mesh_changed_this_frame: bool,
    pub object_lookup: &'a HashMap<Uuid, usize>,
    pub objects_sorted_by_z: &'a Vec<Object>,
    pub vertices: &'a Vec<Vertex>,
    pub indices: &'a Vec<Index>,
}
impl<'a> Scene<'a> {
    pub fn get_object(&self, uuid: &Uuid) -> &Object {
        &self.objects_sorted_by_z[self.object_lookup[uuid]]
    }
}
pub struct FrameAnimationOutput<'a> {
    pub scene: Scene<'a>,
    pub glyph_update_data: Option<GlyphUpdateData<'a>>,
}
impl Animator {
    pub fn new(
        mut animations: Vec<AnimOP>,
        fps: u32,
        camera: Camera,
        text_scale: f32,
    ) -> Result<Self> {
        let first_anim = animations
            .pop()
            .take()
            .context("you need at least one anim op to init anim renderer")?;
        debug!("first_anim: {first_anim:?}");
        Ok(Self {
            camera,
            text_manager: TextManager::new(text_scale),
            fps,
            anim_state: AnimationState::new(first_anim)?,
            animations_left: animations,
            objects: vec![],
            indices: vec![],
            vertices: vec![],
            objects_lookup: HashMap::new(),
            layout_containers: HashMap::new(),
            layout_resizing_in_progress: false,
        })
    }

    pub fn animate_next_frame(&mut self) -> Result<Option<FrameAnimationOutput>> {
        debug!(
            "animate_next_frame- current_anim_state:{:?}",
            self.anim_state
        );

        loop {
            let animation: Animation = self.anim_state.anim_op.clone().try_into()?;

            let mut storage = self.anim_state.storage.clone();

            if self.anim_state.time == 0. {
                let loc_str = animation
                    .location
                    .as_ref()
                    .map(|l| l.to_string())
                    .unwrap_or_default();
                (*animation.start)(self, &mut storage).context(format!(
                    "while running the start function of an animation {loc_str}"
                ))?;
            }
            if let Some(update) = animation.update
                && self.anim_state.time < animation.total_duration
            {
                let t = if self.anim_state.time + 1. / self.fps as f32 >= animation.total_duration {
                    1.0
                } else {
                    self.anim_state.time / animation.total_duration
                };
                let loc_str = animation
                    .location
                    .as_ref()
                    .map(|l| l.to_string())
                    .unwrap_or_default();
                (*update)(self, animation.curve.apply(t), &mut storage).context(format!(
                    "while running the update function of an animation {loc_str}"
                ))?;
                self.anim_state.time += 1. / self.fps as f32;

                self.anim_state.storage = storage;
                break;
            } else {
                self.anim_state.storage = storage;
                match self.animations_left.pop() {
                    Some(op) => {
                        debug!("handle next animation");
                        self.anim_state =
                            AnimationState::new(op).context("while setting up a new anim op")?
                    }
                    None => {
                        debug!("no animations left");
                        return Ok(None);
                    }
                }
            }
        }

        self.update_object_matrix_cache();
        self.sort_objects_by_z();

        let scene = Scene {
            mesh_changed_this_frame: true,
            camera: &self.camera,
            object_lookup: &self.objects_lookup,
            indices: &self.indices,
            objects_sorted_by_z: &self.objects,
            vertices: &self.vertices,
        };

        Ok(Some(FrameAnimationOutput {
            scene,
            glyph_update_data: self.text_manager.atlas.get_glyph_update_data(),
        }))
    }
    pub fn sort_objects_by_z(&mut self) {
        self.objects.sort_by(|a, b| a.z_pos().total_cmp(&b.z_pos()));
        self.objects_lookup = self
            .objects
            .iter()
            .enumerate()
            .map(|(i, obj)| (obj.anim_data.transform().uuid, i))
            .collect();
    }
    pub fn update_object_matrix_cache(&mut self) {
        self.objects
            .iter_mut()
            .for_each(|obj| obj.render_data.world_matrix_cache = Mat4::ZERO);
        let objects = self.objects_lookup.clone();
        for uuid in objects.keys() {
            self.get_object_mut(uuid)
                .unwrap()
                .render_data
                .world_matrix_cache = self.get_object_world_matrix(&uuid).unwrap();
        }
    }

    pub fn check_cycle(&self, child_uuid: &Uuid, parent_uuid: &Uuid) -> Result<()> {
        let mut current = *parent_uuid;
        loop {
            if current == *child_uuid {
                bail!(
                    "circular parent reference: {} would be an ancestor of {}",
                    child_uuid,
                    parent_uuid
                );
            }
            let Some(&idx) = self.objects_lookup.get(&current) else {
                return Ok(());
            };
            let Some(next) = self.objects[idx].anim_data.transform().parent else {
                return Ok(());
            };
            current = next;
        }
    }

    pub(crate) fn get_object(&self, uuid: &Uuid) -> Result<&Object> {
        let index = self
            .objects_lookup
            .get(uuid)
            .with_context(|| format!("there was no object with uuid {uuid}"))?;

        self.objects
            .get(*index)
            .context("index from the object lookup was out of bounds")
    }
    pub(crate) fn get_object_mut(&mut self, uuid: &Uuid) -> Result<&mut Object> {
        let index = self
            .objects_lookup
            .get(uuid)
            .with_context(|| format!("there was no object with uuid {uuid}"))?;
        self.objects
            .get_mut(*index)
            .context("index from the object lookup was out of bounds ")
    }

    pub fn get_object_world_matrix(&self, uuid: &Uuid) -> Result<Mat4> {
        const Z_CHILD_CHANGE: f32 = 0.1;
        let obj = self.get_object(uuid)?;
        if obj.transform().parent.unwrap_or(Uuid::nil()) == *uuid {
            bail!("parent == child");
        }

        if obj.render_data.world_matrix_cache != Mat4::ZERO {
            Ok(obj.render_data.world_matrix_cache)
        } else {
            let transf = obj.transform();
            let local = transf.get_local_matrix();
            match transf.parent {
                Some(parent_id) => {
                    let mut parent_matrix = self.get_object_world_matrix(&parent_id)?;
                    parent_matrix.w_axis.z += Z_CHILD_CHANGE;
                    Ok(parent_matrix * local)
                }
                None => Ok(local),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3};

    fn make_rect_obj(uuid: Uuid, parent: Option<Uuid>) -> Object {
        Object {
            anim_data: AnimObj(Box::new(Rectangle {
                size: Vec2::new(10.0, 10.0),
                corner_radius: 0.0,
                color: Color::WHITE,
                transform: Transform {
                    uuid,
                    parent,
                    position: Vec3::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                    layout_container: None,
                    world_uniform: None,
                },
            })),
            render_data: ObjectRenderData {
                world_matrix_cache: Mat4::ZERO,
                vertices_base_index: 0,
                vertices: vec![],
                indices_base_index: 0,
                indices_count: 0,
                pipeline: PipelineKind::Shape,
                object_bind_groups: vec![],
            },
        }
    }

    fn make_animator() -> Animator {
        let anim = AnimOP::Wait(0.0);
        let camera = Camera::new(Vec2::new(100.0, 100.0), Vec2::ZERO, 1.0);
        Animator::new(vec![anim], 60, camera, 1.0).unwrap()
    }

    #[test]
    fn check_cycle_self_reference() {
        let mut animator = make_animator();
        let uuid = Uuid::new_v4();
        animator.objects.push(make_rect_obj(uuid, Some(uuid)));
        animator.objects_lookup.insert(uuid, 0);
        assert!(animator.check_cycle(&uuid, &uuid).is_err());
    }

    #[test]
    fn check_cycle_no_parent_ok() {
        let mut animator = make_animator();
        let uuid_a = Uuid::new_v4();
        let uuid_b = Uuid::new_v4();
        animator.objects.push(make_rect_obj(uuid_a, None));
        animator.objects_lookup.insert(uuid_a, 0);
        animator.objects.push(make_rect_obj(uuid_b, Some(uuid_a)));
        animator.objects_lookup.insert(uuid_b, 1);
        assert!(animator.check_cycle(&uuid_a, &uuid_b).is_err());
    }

    #[test]
    fn check_cycle_unrelated_ok() {
        let mut animator = make_animator();
        let uuid_a = Uuid::new_v4();
        let uuid_b = Uuid::new_v4();
        animator.objects.push(make_rect_obj(uuid_a, None));
        animator.objects_lookup.insert(uuid_a, 0);
        assert!(animator.check_cycle(&uuid_a, &uuid_b).is_ok());
        assert!(animator.check_cycle(&uuid_b, &Uuid::new_v4()).is_ok());
    }

    #[test]
    fn check_cycle_transitive_cycle() {
        let mut animator = make_animator();
        let uuid_a = Uuid::new_v4();
        let uuid_b = Uuid::new_v4();
        let uuid_c = Uuid::new_v4();
        animator.objects.push(make_rect_obj(uuid_a, Some(uuid_c)));
        animator.objects_lookup.insert(uuid_a, 0);
        animator.objects.push(make_rect_obj(uuid_b, Some(uuid_a)));
        animator.objects_lookup.insert(uuid_b, 1);
        animator.objects.push(make_rect_obj(uuid_c, Some(uuid_b)));
        animator.objects_lookup.insert(uuid_c, 2);
        assert!(animator.check_cycle(&uuid_c, &uuid_b).is_err());
    }

    #[test]
    fn check_cycle_nonexistent_parent_ok() {
        let mut animator = make_animator();
        let uuid = Uuid::new_v4();
        let nonexistent = Uuid::new_v4();
        animator.objects.push(make_rect_obj(uuid, None));
        animator.objects_lookup.insert(uuid, 0);
        assert!(animator.check_cycle(&uuid, &nonexistent).is_ok());
    }
}
