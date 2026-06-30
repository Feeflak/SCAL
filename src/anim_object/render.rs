use std::collections::HashMap;

use log::debug;
use uuid::Uuid;
use wgpu::TextureFormat;

use crate::{
    anim_object::{
        AnimObject, Transform,
        primitive_shapes::{Square, create_shape_pipeline},
        text::Text,
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
fn generate_square_mesh_data(square: &Square) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
    let (w, h) = square.size;

    let hw = w * 0.5;
    let hh = h * 0.5;

    let color = [square.color.0, square.color.1, square.color.2];

    let vertices = vec![
        Vertex {
            position: [-hw, -hh],
            color,
        },
        Vertex {
            position: [hw, -hh],
            color,
        },
        Vertex {
            position: [hw, hh],
            color,
        },
        Vertex {
            position: [-hw, hh],
            color,
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices, PipelineKind::Shape)
}
impl AnimObject {
    pub fn generate_mesh_data(&self) -> (Vec<Vertex>, Vec<Index>, PipelineKind) {
        match self {
            AnimObject::Text(text, _) => todo!(), // Skip implementation for now
            AnimObject::Square(square, _) => generate_square_mesh_data(square),
        }
    }
}
impl Animator {
    pub fn add_anim_object(&mut self, obj: AnimObject) {
        let (render_data, mut indices) = {
            let (default_vertices, mut indices, pipeline) = obj.generate_mesh_data();

            let vertex_base = self.vertices.len();
            let index_base = self.indices.len();

            for index in &mut indices {
                *index += vertex_base as u32;
            }

            (
                ObjectRenderData {
                    pipeline,
                    vertices_base_index: vertex_base,
                    base_vertices: default_vertices.clone(),
                    indices_base_index: index_base,
                    indices_count: indices.len(),
                },
                indices,
            )
        };
        let mut vertices = render_data.transform_updated_vertices(obj.transform());

        self.vertices.append(&mut vertices);
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
            .drain(data.vertices_base_index..data.vertices_base_index + data.base_vertices.len());

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
pub struct ObjectRenderData {
    pub vertices_base_index: usize,
    pub base_vertices: Vec<Vertex>,
    pub indices_base_index: usize,
    pub indices_count: usize,
    pub pipeline: PipelineKind,
}
impl ObjectRenderData {
    pub fn transform_updated_vertices(&self, transform: &Transform) -> Vec<Vertex> {
        let cos = transform.rotation.cos();
        let sin = transform.rotation.sin();

        let mut vertices = self.base_vertices.clone();
        for vert in vertices.iter_mut() {
            let x = vert.position[0] * transform.scale;
            let y = vert.position[1] * transform.scale;

            let rx = x * cos - y * sin;
            let ry = x * sin + y * cos;
            vert.position = [rx + transform.pos.0, ry + transform.pos.1];
        }
        vertices
    }
}

pub fn get_pipelines(device: &wgpu::Device) -> HashMap<PipelineKind, wgpu::RenderPipeline> {
    const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    HashMap::from([(PipelineKind::Shape, create_shape_pipeline(device, FORMAT))])
}
