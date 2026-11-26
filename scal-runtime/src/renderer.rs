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
use scal_core::Color;
use std::collections::HashMap;
use uuid::Uuid;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

use crate::anim_object::render::{PipelineData, PipelineKind};
use crate::animator::{Object, Scene};
use glam::{Mat4, Vec2};

pub type Index = u32;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: Vec2,
    pub color: Color,
    pub uv: Vec2,
}

/// Uniform data sent to the GPU for each object transform.
/// Includes the world matrix and an optional axis-aligned clip rectangle in
/// the object's local space. When the clip bounds are very large, clipping is
/// effectively disabled.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectTransformUniform {
    pub matrix: [[f32; 4]; 4],
    pub clip_rect: [f32; 4],
    pub _padding: [f32; 4],
}

impl Default for ObjectTransformUniform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY.to_cols_array_2d(),
            clip_rect: [-1e9, -1e9, 1e9, 1e9],
            _padding: [0.0; 4],
        }
    }
}

pub struct ObjectTransformData {
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}
struct CachedBindGroups {
    groups: Vec<BindGroup>,
    /// `Renderer::generation` when the entry was built or last reused.
    generation: u64,
    last_access: u64,
}

pub struct Renderer {
    pub object_transform_data_lookup: HashMap<Uuid, ObjectTransformData>,
    /// Bind groups per object, with the `content_version` they were built for.
    pub per_object_bind_groups: HashMap<Uuid, (u64, Vec<BindGroup>)>,
    /// Bind groups shared between objects with the same content (see
    /// `AnimObjectTrait::bind_group_cache_key`). Unlike the uuid keyed maps this survives a
    /// reload, so unchanged images, svgs and equations are not rebuilt.
    content_bind_groups: HashMap<String, CachedBindGroups>,
    /// Counts reloads; tells which `content_bind_groups` entries are still in use.
    generation: u64,
    /// Counts cache accesses to find the least recently used entry.
    access_counter: u64,
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
            content_bind_groups: HashMap::new(),
            generation: 0,
            access_counter: 0,
        }
    }

    /// Call when the animation is replaced. Object uuids are regenerated on every reload, so
    /// everything keyed by uuid is unreachable and dropped. Content cached bind groups are kept,
    /// except the ones the previous animation never used.
    pub(crate) fn begin_reload(&mut self) {
        let transforms = self.object_transform_data_lookup.len();
        self.object_transform_data_lookup.clear();
        self.per_object_bind_groups.clear();
        let before = self.content_bind_groups.len();
        let generation = self.generation;
        self.content_bind_groups
            .retain(|_, cached| cached.generation >= generation);
        self.generation += 1;
        debug!(
            "renderer reset for reload: dropped {transforms} object transform buffers, kept {} of {before} cached bind group sets",
            self.content_bind_groups.len()
        );
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
                self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
                self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

            let clip_rect = obj
                .anim_data
                .transform()
                .clip_rect
                .unwrap_or([-1e9, -1e9, 1e9, 1e9]);
            let uniform = ObjectTransformUniform {
                matrix: obj.render_data.world_matrix_cache.to_cols_array_2d(),
                clip_rect,
                _padding: [0.0; 4],
            };
            queue.write_buffer(&transform_data.buffer, 0, bytemuck::bytes_of(&uniform));
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
            let version = obj.anim_data.content_version();
            if self
                .per_object_bind_groups
                .get(&uuid)
                .is_some_and(|(built_for, _)| *built_for == version)
            {
                continue;
            }
            let cache_key = obj.anim_data.bind_group_cache_key();
            if let Some(key) = &cache_key {
                self.access_counter += 1;
                if let Some(cached) = self.content_bind_groups.get_mut(key) {
                    cached.last_access = self.access_counter;
                    cached.generation = self.generation;
                    let groups = cached.groups.clone();
                    debug!("bind groups of object {uuid} reused from cache ({key})");
                    self.per_object_bind_groups.insert(uuid, (version, groups));
                    continue;
                }
            }
            if let Some(loader) = obj.anim_data.bind_group_loader() {
                let started = std::time::Instant::now();
                let groups = loader(device, queue);
                let label = cache_key.as_deref().unwrap_or("uncached");
                if groups.is_empty() {
                    log::error!("could not build the bind groups of object {uuid} ({label})");
                } else {
                    debug!(
                        "built bind groups of object {uuid} in {:.1?} ({label})",
                        started.elapsed()
                    );
                }
                if let Some(key) = cache_key
                    && !groups.is_empty()
                {
                    self.insert_cached_bind_groups(key, groups.clone());
                }
                self.per_object_bind_groups.insert(uuid, (version, groups));
            }
        }

        Ok(())
    }

    fn insert_cached_bind_groups(&mut self, key: String, groups: Vec<BindGroup>) {
        /// Entries kept (each holds a texture) before the least recently used one is dropped.
        const MAX_CACHED: usize = 256;
        if self.content_bind_groups.len() >= MAX_CACHED
            && let Some(oldest) = self
                .content_bind_groups
                .iter()
                .min_by_key(|(_, cached)| cached.last_access)
                .map(|(key, _)| key.clone())
        {
            self.content_bind_groups.remove(&oldest);
        }
        self.content_bind_groups.insert(
            key,
            CachedBindGroups {
                groups,
                generation: self.generation,
                last_access: self.access_counter,
            },
        );
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
                .map(|(_, v)| v.as_slice())
                .unwrap_or(&[]);
            // A loader ran but produced no bind groups (it failed and logged
            // an error); drawing would trigger a fatal wgpu validation error.
            if object_bind_groups.is_empty()
                && self.per_object_bind_groups.contains_key(object.uuid())
            {
                continue;
            }
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
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<ObjectTransformUniform>() as _,
                ),
            },
            count: None,
        }],
    })
}
fn create_transform_bind_group_and_buffer(device: &wgpu::Device) -> (BindGroup, Buffer) {
    let bind_group_layout = transform_bind_group_layout(device);
    let initial = ObjectTransformUniform::default();
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Transform Buffer"),
        contents: bytemuck::bytes_of(&initial),

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
