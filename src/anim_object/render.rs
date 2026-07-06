use std::collections::HashMap;

use anyhow::{Context, Result};
use glam::{Mat4, Vec2, Vec3, vec2, vec3};
use uuid::Uuid;
use log::debug;
use wgpu::TextureFormat;

use crate::{
    anim_object::{
        object_trait::AnimObj,
        compose::{Alignment, LayoutContainer, LayoutDir},
        image::create_image_pipeline,
        primitive_shapes::{create_shape_pipeline, Rectangle},
        text::{pipeline::create_text_pipeline},
    },
    animator::{Animator, Object},
    renderer::Vertex,
};
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipelineKind {
    Shape,
    Text,
    Image,
}
impl Animator {
    pub fn add_anim_object(&mut self, mut anim_data: AnimObj) -> Result<()> {
        let id = anim_data.uuid();

        if let Some(container) = anim_data.as_any().downcast_ref::<LayoutContainer>() {
            self.layout_containers.insert(container.id, container.clone());
        }

        if let Some(parent_uuid) = anim_data.transform().parent {
            self.check_cycle(&anim_data.transform().uuid, &parent_uuid)?;
        }

        let (render_data, mut indices) = {
            let (vertives, indices, pipeline) =
                anim_data.generate_mesh(&mut self.text_manager);

            let vertex_base = self.vertices.len();
            let index_base = self.indices.len();

            (
                ObjectRenderData {
                    world_matrix_cache: Mat4::ZERO,
                    pipeline,
                    vertices_base_index: vertex_base,
                    vertices: vertives.clone(),
                    indices_base_index: index_base,
                    indices_count: indices.len(),
                    object_bind_groups: vec![],
                },
                indices,
            )
        };

        self.vertices.append(&mut render_data.vertices.clone());
        self.indices.append(&mut indices);

        let obj_idx = self.objects.len();
        self.objects_lookup.insert(id, obj_idx);
        self.objects.push(Object {
            anim_data,
            render_data,
        });

        debug!("add_anim_object- objects:{:?}", self.objects);

        // Reconcile estimated layout positions with actual mesh sizes
        let _ = self.maybe_resize_layout(&id);

        Ok(())
    }

    pub fn regenerate_object_mesh(&mut self, uuid: &Uuid) -> Result<()> {
        let obj_idx = *self
            .objects_lookup
            .get(uuid)
            .context("object not found for mesh regeneration")?;
        let obj = &mut self.objects[obj_idx];

        let (new_vertices_data, new_indices, pipeline) =
            obj.anim_data.generate_mesh(&mut self.text_manager);

        let old_vert_start = obj.render_data.vertices_base_index;
        let old_vert_count = obj.render_data.vertices.len();
        let old_idx_start = obj.render_data.indices_base_index;
        let old_idx_count = obj.render_data.indices_count;

        let new_vert_count = new_vertices_data.len();
        let vert_diff = new_vert_count as isize - old_vert_count as isize;
        let new_idx_count = new_indices.len();
        let idx_diff = new_idx_count as isize - old_idx_count as isize;

        self.vertices
            .splice(old_vert_start..old_vert_start + old_vert_count, new_vertices_data.clone());
        self.indices
            .splice(old_idx_start..old_idx_start + old_idx_count, new_indices);

        obj.render_data.vertices = new_vertices_data;
        obj.render_data.indices_count = new_idx_count;
        obj.render_data.pipeline = pipeline;

        for other in self.objects.iter_mut() {
            if other.render_data.vertices_base_index > old_vert_start {
                other.render_data.vertices_base_index =
                    (other.render_data.vertices_base_index as isize + vert_diff) as usize;
            }
            if other.render_data.indices_base_index > old_idx_start {
                other.render_data.indices_base_index =
                    (other.render_data.indices_base_index as isize + idx_diff) as usize;
            }
        }

        self.maybe_resize_layout(uuid)?;

        Ok(())
    }

    fn maybe_resize_layout(&mut self, child_uuid: &Uuid) -> Result<()> {
        if self.layout_resizing_in_progress {
            return Ok(());
        }
        self.layout_resizing_in_progress = true;

        let result = self.maybe_resize_layout_inner(child_uuid);

        self.layout_resizing_in_progress = false;
        result
    }

    fn maybe_resize_layout_inner(&mut self, child_uuid: &Uuid) -> Result<()> {
        let container_uuid = match self.get_object(child_uuid)?.anim_data.layout_parent() {
            Some(u) => u,
            None => return Ok(()),
        };

        let container = match self.layout_containers.get(&container_uuid) {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        if container.background_uuid == *child_uuid {
            return Ok(());
        }

        let sizes: Vec<Vec2> = match container.child_uuids.iter()
            .map(|id| self.get_object(id).map(|o| o.anim_data.size()))
            .collect::<Result<Vec<_>>>() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };
        let max_w = sizes.iter().map(|s| s.x).fold(0.0f32, f32::max);
        let total_h: f32 = sizes.iter().map(|s| s.y).sum();

        let gaps = container.gap * (container.child_uuids.len() as f32 - 1.0).max(0.0);
        let new_w = (max_w + container.padding_left + container.padding_right).max(container.min_width);
        let new_h = (total_h + container.padding_top + container.padding_bottom + gaps).max(container.min_height);

        let old_bg_size: Vec2;
        let old_bg_pos: Vec3;
        {
            let bg = self.get_object(&container.background_uuid)?;
            if let Some(rect) = bg.anim_data.as_any().downcast_ref::<Rectangle>() {
                old_bg_size = rect.size;
            } else {
                return Ok(());
            }
            old_bg_pos = bg.anim_data.transform().position;
        }

        let dh = new_h - old_bg_size.y;

        // Resize background rect and shift its center down by dh/2 so the top edge stays fixed
        {
            let bg = self.get_object_mut(&container.background_uuid)?;
            if let Some(rect) = bg.anim_data.as_any_mut().downcast_mut::<Rectangle>() {
                rect.size = vec2(new_w, new_h);
            }
            bg.anim_data.transform_mut().position = old_bg_pos + vec3(0.0, dh / 2.0, 0.0);
        }

        // Reposition children according to alignment and their own width.
        // Stretch Rectangle children to fill max_w.
        let content_left = -new_w / 2.0 + container.padding_left;
        let content_right = new_w / 2.0 - container.padding_right;
        let content_bottom = new_h / 2.0 - container.padding_bottom;
        let content_top = -new_h / 2.0 + container.padding_top;

        let mut y = content_top;
        let mut x = content_left;

        for (i, child_id) in container.child_uuids.iter().enumerate() {
            let s = sizes[i];
            let (child_x, child_y) = match container.direction {
                LayoutDir::Column => {
                    let cx = match container.alignment {
                        Alignment::Start => content_left + s.x / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_right - s.x / 2.0,
                    };
                    let cy = y + s.y / 2.0;
                    y += s.y + container.gap;
                    (cx, cy)
                }
                LayoutDir::Row => {
                    let cx = x + s.x / 2.0;
                    let cy = match container.alignment {
                        Alignment::Start => content_bottom - s.y / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_top + s.y / 2.0,
                    };
                    x += s.x + container.gap;
                    (cx, cy)
                }
            };
            let obj = self.get_object_mut(child_id)?;
            let is_stretched = obj.anim_data.as_any().downcast_ref::<Rectangle>().is_some();
            obj.anim_data.transform_mut().position.x = if is_stretched {
                0.0
            } else {
                child_x
            };
            obj.anim_data.transform_mut().position.y = child_y;

            let mut regen = false;
            if let Some(rect) = obj.anim_data.as_any_mut().downcast_mut::<Rectangle>() {
                rect.size.x = new_w;
                regen = true;
            }
            if regen {
                drop(obj);
                self.regenerate_object_mesh(child_id)?;
                // If this Rectangle is the background of a nested layout, relayout its children
                if let Some(nested) = self.layout_containers.values()
                    .find(|c| c.background_uuid == *child_id).cloned()
                {
                    self.relayout_container_children(&nested)?;
                }
            }
        }

        self.regenerate_object_mesh(&container.background_uuid)?;

        Ok(())
    }

    fn relayout_container_children(&mut self, container: &LayoutContainer) -> Result<()> {
        let bg_size: Vec2;
        {
            let bg = self.get_object(&container.background_uuid)?;
            let Some(rect) = bg.anim_data.as_any().downcast_ref::<Rectangle>() else {
                return Ok(());
            };
            bg_size = rect.size;
        }

        let sizes: Vec<Vec2> = match container.child_uuids.iter()
            .map(|id| self.get_object(id).map(|o| o.anim_data.size()))
            .collect::<Result<Vec<_>>>() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };
        let max_w = bg_size.x - container.padding_left - container.padding_right;

        let content_left = -bg_size.x / 2.0 + container.padding_left;
        let content_right = bg_size.x / 2.0 - container.padding_right;
        let content_bottom = bg_size.y / 2.0 - container.padding_bottom;
        let content_top = -bg_size.y / 2.0 + container.padding_top;

        let mut y = content_top;
        let mut x = content_left;

        for (i, child_id) in container.child_uuids.iter().enumerate() {
            let s = sizes[i];
            let (child_x, child_y) = match container.direction {
                LayoutDir::Column => {
                    let cx = match container.alignment {
                        Alignment::Start => content_left + s.x / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_right - s.x / 2.0,
                    };
                    let cy = y + s.y / 2.0;
                    y += s.y + container.gap;
                    (cx, cy)
                }
                LayoutDir::Row => {
                    let cx = x + s.x / 2.0;
                    let cy = match container.alignment {
                        Alignment::Start => content_bottom - s.y / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_top + s.y / 2.0,
                    };
                    x += s.x + container.gap;
                    (cx, cy)
                }
            };
            let obj = self.get_object_mut(child_id)?;
            obj.anim_data.transform_mut().position = vec3(child_x, child_y, 0.0);
        }

        Ok(())
    }

    pub fn remove_anim_object(&mut self, obj: AnimObj) {
        let id = obj.uuid();

        let Some(object_index) = self.objects_lookup.remove(&id) else {
            return;
        };

        let obj = self.objects.remove(object_index);
        {
            let data = obj.render_data;
            self.vertices
                .drain(data.vertices_base_index..data.vertices_base_index + data.vertices.len());

            self.indices
                .drain(data.indices_base_index..data.indices_base_index + data.indices_count);
        }

        // rebuild lookup because offsets shifted
        self.objects_lookup.clear();

        for (i, obj) in self.objects.iter().enumerate() {
            self.objects_lookup.insert(*obj.uuid(), i);
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct ObjectRenderData {
    pub world_matrix_cache: Mat4,
    pub vertices_base_index: usize,
    pub vertices: Vec<Vertex>,
    pub indices_base_index: usize,
    pub indices_count: usize,
    pub pipeline: PipelineKind,
    pub object_bind_groups: Vec<wgpu::BindGroup>,
}
pub(crate) struct PipelineData {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
}

pub(crate) fn get_pipelines(device: &wgpu::Device) -> HashMap<PipelineKind, PipelineData> {
    const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    HashMap::from([
        (PipelineKind::Text, create_text_pipeline(device, FORMAT)),
        (PipelineKind::Shape, create_shape_pipeline(device, FORMAT)),
        (PipelineKind::Image, create_image_pipeline(device, FORMAT)),
    ])
}
