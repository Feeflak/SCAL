//! GPU rendering pipeline and buffer management.
//!
//! # Pipeline layout
//!
//! | Bind index | Resource          | Scope              |
//! |------------|-------------------|--------------------|
//! | 0          | Camera uniform    | Per-frame          |
//! | 1          | Transform uniform | Per-object         |
//! | 2+         | Object-specific   | Per-pipeline       |
//!
//! # Render order
//!
//! Objects are sorted by z (back-to-front) in the [`Animator`]. The renderer
//! iterates the sorted list, switching pipeline (Shape / Text / Image) as needed.
//!
//! # Buckets
//!
//! There are no explicit bucket structures. Each [`PipelineKind`] maps to
//! exactly one [`wgpu::RenderPipeline`] and a set of shared bind groups.
//! During drawing the same pipeline is reused for all objects that share it,
//! which gives implicit batching.
//!
//! # Buffer growth
//!
//! Vertex and index buffers are re-created when the mesh size changes
//! (detected by comparing the current vs previous length). This is cheap
//! because mesh updates are infrequent after initial object instantiation.
//!
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use uuid::Uuid;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

use crate::anim_object::render::{PipelineData, PipelineKind};
use crate::animator::{Object, Scene};
use crate::projection::Camera;
use crate::types::Color;
use glam::{Mat4, Vec2};

pub type Index = u32;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: Vec2,
    pub color: Color,
    pub uv: Vec2,
}

pub struct ObjectTransformData {
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}
pub struct Renderer {
    pub object_transform_data_lookup: HashMap<Uuid, ObjectTransformData>,
    pub per_object_bind_groups: HashMap<Uuid, Vec<BindGroup>>,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: wgpu::Buffer,

    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: usize,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_size: usize,
}

impl Renderer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let camera_bind_group_layout = camera_bind_group_layout(device);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&Mat4::ZERO),

            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera BG"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let default_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        // Camera and transform buffers need to be already created in the right way, other ones will be recreated
        // after each frame that added a new object- more indices, vertices

        Self {
            vertex_buffer: default_buffer.clone(),
            vertex_buffer_size: 0,
            index_buffer: default_buffer.clone(),
            index_buffer_size: 0,
            camera_bind_group,
            camera_buffer,
            object_transform_data_lookup: HashMap::new(),
            per_object_bind_groups: HashMap::new(),
        }
    }

    pub(crate) fn update_render_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
    ) -> Result<()> {
        if scene.mesh_changed_this_frame {
            let indices = scene.indices;
            let prev_idx_size = self.index_buffer_size;
            if prev_idx_size != indices.len() {
                self.index_buffer_size = indices.len();
                self.index_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Object Index Buffer"),
                        contents: bytemuck::cast_slice(indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    });
                debug!(
                    "Index buffer resized: old={} new={}",
                    prev_idx_size,
                    indices.len()
                );
            } else {
                queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
            }
            let vertices = scene.vertices;
            let prev_vert_size = self.vertex_buffer_size;
            if prev_vert_size != vertices.len() {
                self.vertex_buffer_size = vertices.len();
                self.vertex_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Object Vertex Buffer"),
                        contents: bytemuck::cast_slice(vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                debug!(
                    "Vertex buffer resized: old={} new={} ({} bytes)",
                    prev_vert_size,
                    vertices.len(),
                    vertices.len() * std::mem::size_of::<Vertex>(),
                );
            } else {
                queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
            }
        }

        for obj in scene.objects_sorted_by_z {
            let transf = obj.anim_data.transform();
            let transform_data = self
                .object_transform_data_lookup
                .entry(transf.uuid)
                .or_insert_with(|| {
                    let (bind_group, buffer) = create_transform_bind_group_and_buffer(device);
                    ObjectTransformData { bind_group, buffer }
                });

            queue.write_buffer(
                &transform_data.buffer,
                0,
                bytemuck::bytes_of(&obj.render_data.world_matrix_cache),
            );
        }

        if scene.camera.dirty {
            queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::bytes_of(&scene.camera.get_matrix()),
            );
        }

        for obj in scene.objects_sorted_by_z {
            let uuid = *obj.uuid();
            if self.per_object_bind_groups.contains_key(&uuid) {
                continue;
            }
            if let Some(loader) = obj.anim_data.bind_group_loader() {
                let groups = loader(device, queue);
                self.per_object_bind_groups.insert(uuid, groups);
            }
        }

        Ok(())
    }

    pub(crate) fn draw_objects(
        &self,
        render_pass: &mut wgpu::RenderPass,
        pipelines: &HashMap<PipelineKind, PipelineData>,
        objects_sorted_by_z: &[Object],
    ) {
        const CAMERA_BIND_INDEX: u32 = 0;
        const TRANSFORM_BIND_INDEX: u32 = 1;
        const OTHER_BINDING_OFFSET: u32 = 2;
        if self.vertex_buffer_size > 0 {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        }
        if self.index_buffer_size > 0 {
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }
        render_pass.set_bind_group(CAMERA_BIND_INDEX, &self.camera_bind_group, &[]);
        let mut current_pipeline: Option<PipelineKind> = None;
        for object in objects_sorted_by_z {
            if current_pipeline != Some(object.render_data.pipeline) {
                current_pipeline = Some(object.render_data.pipeline);
                let pipeline_data = &pipelines[&object.render_data.pipeline];
                render_pass.set_pipeline(&pipeline_data.pipeline);
                for (i, bind_group) in pipeline_data.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(OTHER_BINDING_OFFSET + i as u32, bind_group, &[]);
                }
            }
            let Some(transform_data) = self.object_transform_data_lookup.get(object.uuid()) else {
                log::warn!(
                    "draw_objects: missing transform for object {} — skipping",
                    object.uuid()
                );
                continue;
            };
            let bind_group = &transform_data.bind_group;

            render_pass.set_bind_group(TRANSFORM_BIND_INDEX, bind_group, &[]);

            let object_bind_groups = self
                .per_object_bind_groups
                .get(object.uuid())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            for (i, bg) in object_bind_groups.iter().enumerate() {
                render_pass.set_bind_group(OTHER_BINDING_OFFSET + i as u32, bg, &[]);
            }

            if object.render_data.indices_count > 0 {
                render_pass.draw_indexed(
                    object.render_data.indices_base_index as u32
                        ..(object.render_data.indices_base_index + object.render_data.indices_count)
                            as u32,
                    object.render_data.vertices_base_index as i32,
                    0..1,
                );
            }
        }
    }
}

pub(crate) fn camera_bind_group_layout(device: &wgpu::Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[[f32; 4]; 4]>() as _),
            },
            count: None,
        }],
    })
}
pub(crate) fn transform_bind_group_layout(device: &wgpu::Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[[f32; 4]; 4]>() as _),
            },
            count: None,
        }],
    })
}
fn create_transform_bind_group_and_buffer(device: &wgpu::Device) -> (BindGroup, Buffer) {
    let bind_group_layout = camera_bind_group_layout(device);
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Transform Buffer"),
        contents: bytemuck::bytes_of(&Mat4::ZERO),

        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("transform BG"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (bind_group, buffer)
}

pub(crate) fn draw_scene(
    render_pass: &mut wgpu::RenderPass,
    pipelines: &HashMap<PipelineKind, PipelineData>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: Scene,
    renderer: &mut Renderer,
) -> Result<()> {
    if scene.objects_sorted_by_z.len() == 0 {
        debug!("No objects, skipping drawing buckets");
        return Ok(());
    }

    renderer.update_render_buffers(device, queue, &scene)?;
    renderer.draw_objects(render_pass, pipelines, scene.objects_sorted_by_z);
    Ok(())
}

#[derive(Clone, Copy)]
pub struct RenderingSettings {
    pub camera: Camera,
    pub background_color: Color,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
    pub text_resolution_multiplier: f32,
}
