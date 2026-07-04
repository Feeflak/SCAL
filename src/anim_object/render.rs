use std::collections::HashMap;

use anyhow::{Context, Result};
use glam::Mat4;
use uuid::Uuid;
use log::debug;
use wgpu::TextureFormat;

use crate::{
    anim_object::{
        object_trait::{AnimObj, AnimObjectTrait},
        image::create_image_pipeline,
        primitive_shapes::create_shape_pipeline,
        text::{TextManager, pipeline::create_text_pipeline},
    },
    animator::{Animator, Object},
    renderer::{Index, Vertex},
};
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipelineKind {
    Shape,
    Text,
    Image,
}
impl Animator {
    pub fn add_anim_object(&mut self, mut anim_data: AnimObj) -> Result<()> {
        if let Some(parent_uuid) = anim_data.transform().parent {
            self.check_cycle(&anim_data.transform().uuid, &parent_uuid)?;
        }

        let (render_data, mut indices) = {
            let (vertives, mut indices, pipeline) =
                anim_data.generate_mesh(&mut self.text_manager);

            let vertex_base = self.vertices.len();
            let index_base = self.indices.len();

            for index in &mut indices {
                *index += vertex_base as u32;
            }

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

        let id = anim_data.uuid();

        let obj_idx = self.objects.len();
        self.objects_lookup.insert(id, obj_idx);
        self.objects.push(Object {
            anim_data,
            render_data,
        });

        debug!("add_anim_object- objects:{:?}", self.objects);
        Ok(())
    }

    pub fn regenerate_object_mesh(&mut self, uuid: &Uuid) -> Result<()> {
        let obj_idx = *self
            .objects_lookup
            .get(uuid)
            .context("object not found for mesh regeneration")?;
        let obj = &mut self.objects[obj_idx];

        let (new_vertices_data, mut new_indices, pipeline) =
            obj.anim_data.generate_mesh(&mut self.text_manager);

        let old_vert_start = obj.render_data.vertices_base_index;
        let old_vert_count = obj.render_data.vertices.len();
        let old_idx_start = obj.render_data.indices_base_index;
        let old_idx_count = obj.render_data.indices_count;

        let new_vert_count = new_vertices_data.len();
        let vert_diff = new_vert_count as isize - old_vert_count as isize;
        let new_idx_count = new_indices.len();
        let idx_diff = new_idx_count as isize - old_idx_count as isize;

        for idx in &mut new_indices {
            *idx += old_vert_start as u32;
        }

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
