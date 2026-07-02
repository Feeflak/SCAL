use std::collections::HashMap;

use glam::Vec2;
use log::debug;
use wgpu::TextureFormat;

use crate::{
    anim_object::{
        AnimObject, Transform,
        primitive_shapes::{create_shape_pipeline, mesh::generate_square_mesh_data},
        text::{
            TextManager, code::mesh::generate_code_mesh, mesh::generate_text_mesh,
            pipeline::create_text_pipeline,
        },
    },
    animator::Animator,
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
            AnimObject::Square(square, _) => generate_square_mesh_data(square),
        }
    }
}
impl Animator {
    pub fn add_anim_object(&mut self, mut obj: AnimObject) {
        let (render_data, mut indices) = {
            let (vertives, mut indices, pipeline) = obj.generate_mesh_data(&mut self.text_manager);

            let vertex_base = self.vertices.len();
            let index_base = self.indices.len();

            for index in &mut indices {
                *index += vertex_base as u32;
            }

            (
                ObjectRenderData {
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

        let id = obj.transform().uuid;

        self.objects_lookup.insert(id, self.objects.len());
        self.objects.push((obj, render_data));

        debug!("add_anim_object- objects:{:?}", self.objects);
    }

    pub fn remove_anim_object(&mut self, obj: AnimObject) {
        let id = obj.transform().uuid;

        let Some(object_index) = self.objects_lookup.remove(&id) else {
            return;
        };

        let (_, data) = self.objects.remove(object_index);

        self.vertices
            .drain(data.vertices_base_index..data.vertices_base_index + data.vertices.len());

        self.indices
            .drain(data.indices_base_index..data.indices_base_index + data.indices_count);

        // rebuild lookup because offsets shifted
        self.objects_lookup.clear();

        for (i, (obj, _)) in self.objects.iter().enumerate() {
            self.objects_lookup.insert(obj.transform().uuid, i);
        }
    }
}
#[derive(Debug)]
pub(crate) struct ObjectRenderData {
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
