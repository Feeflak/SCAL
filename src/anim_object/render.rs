use std::collections::HashMap;

use anyhow::Result;
use glam::{Mat4, Vec2};
use log::debug;
use wgpu::TextureFormat;

use crate::{
    anim_object::{
        AnimObject, Transform,
        primitive_shapes::{
            create_shape_pipeline,
            mesh::{
                generate_circle_mesh_data, generate_polygon_mesh_data,
                generate_rectangle_mesh_data,
            },
        },
        text::{
            TextManager, code::mesh::generate_code_mesh, mesh::generate_text_mesh,
            pipeline::create_text_pipeline,
        },
    },
    animator::{Animator, Object},
    renderer::{Index, Vertex},
};
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipelineKind {
    Shape,
    Text,
    // later: Sprite, Mesh3D, Particle, etc.
}
impl AnimObject {
    pub fn generate_mesh_data(
        &mut self,
        text_manager: &mut TextManager,
    ) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
        match self {
            AnimObject::Code(code, transform) => {
                generate_code_mesh(text_manager, transform.uuid, code)
            }
            AnimObject::Text(text, _) => generate_text_mesh(text_manager, &text),
            AnimObject::Square(square, _) => generate_rectangle_mesh_data(square),
            AnimObject::Circle(circle, _) => generate_circle_mesh_data(circle),
            AnimObject::Polygon(polygon, _) => generate_polygon_mesh_data(polygon),
        }
    }
}
impl Animator {
    pub fn add_anim_object(&mut self, mut anim_data: AnimObject) -> Result<()> {
        if let Some(parent_uuid) = anim_data.transform().parent {
            self.check_cycle(&anim_data.transform().uuid, &parent_uuid)?;
        }

        let (render_data, mut indices) = {
            let (vertives, mut indices, pipeline) =
                anim_data.generate_mesh_data(&mut self.text_manager);

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
                },
                indices,
            )
        };

        self.vertices.append(&mut render_data.vertices.clone());
        self.indices.append(&mut indices);

        let id = anim_data.transform().uuid;

        self.objects_lookup.insert(id, self.objects.len());
        self.objects.push(Object {
            anim_data,
            render_data,
        });

        debug!("add_anim_object- objects:{:?}", self.objects);
        Ok(())
    }

    pub fn remove_anim_object(&mut self, obj: AnimObject) {
        let id = obj.transform().uuid;

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
    ])
}
