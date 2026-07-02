use log::{debug, info};
use std::collections::HashMap;
use uuid::Uuid;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

use crate::anim_object::AnimObject;
use crate::anim_object::render::{ObjectRenderData, PipelineData, PipelineKind};
use crate::animator::Scene;
use crate::projection::Camera;
use crate::types::Color;
use glam::{Mat4, Vec2};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct RenderNodeId(pub Uuid);
#[derive(Clone)]
pub struct RenderItem<'a> {
    pub object: &'a AnimObject,
    pub data: &'a ObjectRenderData,
    pub pipeline: PipelineKind,
    pub depth: f32,
}

pub struct RenderBuckets<'a> {
    pub buckets: HashMap<PipelineKind, Vec<RenderItem<'a>>>,
}

pub type Index = u32;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Color,
    pub uv: Vec2,
}

pub struct Renderer {
    pub per_object_transform_bind_groups: HashMap<Uuid, (BindGroup, Buffer)>,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: wgpu::Buffer,

    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: usize,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_size: usize,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
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
            contents: &vec![],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        // Camera and transform buffers need to be already created in the right way, other ones will be recreated
        // after each frame that added a new object- more indices, vertices

        Self {
            vertex_buffer: default_buffer.clone(),
            vertex_buffer_size: 0,
            index_buffer: default_buffer.clone(),
            index_buffer_size: 0,
            camera_bind_group: camera_bind_group,
            camera_buffer: camera_buffer,
            per_object_transform_bind_groups: HashMap::new(),
        }
    }
    pub fn update_text_glyphs() {}

    pub fn update_render_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
    ) {
        if scene.mesh_changed_this_frame {
            {
                let indices = scene.indices;
                if self.index_buffer_size != indices.len() {
                    self.index_buffer_size = indices.len();
                    self.index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Object Index Buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
                } else {
                    queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
                }
            }
            {
                let vertices = scene.vertices;
                if self.vertex_buffer_size != vertices.len() {
                    self.vertex_buffer_size = vertices.len();
                    self.vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Object Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });
                } else {
                    queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
                }
            }
        }

        {
            for obj in scene.objects {
                let transf = obj.0.transform();
                if transf.changed_this_frame {
                    let buffer = match self.per_object_transform_bind_groups.get(&transf.uuid) {
                        Some((_, buffer)) => buffer,
                        None => {
                            let (bind_group, buffer) =
                                create_transform_bind_group_and_buffer(device);
                            self.per_object_transform_bind_groups
                                .insert(transf.uuid, (bind_group, buffer));
                            &self
                                .per_object_transform_bind_groups
                                .get(&transf.uuid)
                                .unwrap()
                                .1
                        }
                    };
                    info!(
                        "write transform buffer for: {obj:?}, matrix: {}",
                        &transf.get_matrix()
                    );

                    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&transf.get_matrix()));
                }
            }
        }

        {
            if scene.camera.dirty {
                queue.write_buffer(
                    &self.camera_buffer,
                    0,
                    bytemuck::bytes_of(&scene.camera.get_matrix()),
                );
            }
        }
    }

    pub fn draw_buckets(
        &self,
        render_pass: &mut wgpu::RenderPass,
        pipelines: &HashMap<PipelineKind, PipelineData>,
        buckets: RenderBuckets,
    ) {
        const CAMERA_BIND_INDEX: u32 = 0;
        const TRANSFORM_BIND_INDEX: u32 = 1;
        const OTHER_BINDING_OFFSET: u32 = 2;
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(CAMERA_BIND_INDEX, &self.camera_bind_group, &[]);

        for (pipeline_kind, mut items) in buckets.buckets {
            items.sort_by(|a, b| a.depth.total_cmp(&b.depth));

            let pipeline_data = &pipelines[&pipeline_kind];
            render_pass.set_pipeline(&pipeline_data.pipeline);

            for (i, bind_group) in pipeline_data.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(OTHER_BINDING_OFFSET + i as u32, bind_group, &[]);
            }
            for item in items {
                let bind_group = &self
                    .per_object_transform_bind_groups
                    .get(&item.object.transform().uuid)
                    .expect("there was no transform matrix in the lookup for a transform, probably someone forgot to mark it as dirty initially.")
                    .0;

                render_pass.set_bind_group(TRANSFORM_BIND_INDEX, bind_group, &[]);
                // info!(
                //     "item.data.indices_base_index- {}",
                //     item.data.indices_base_index
                // );

                render_pass.draw_indexed(
                    item.data.indices_base_index as u32
                        ..(item.data.indices_base_index + item.data.indices_count) as u32,
                    0,
                    0..1,
                );
            }
        }
    }
}

pub fn camera_bind_group_layout(device: &wgpu::Device) -> BindGroupLayout {
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
pub fn transform_bind_group_layout(device: &wgpu::Device) -> BindGroupLayout {
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

fn collect_render_items<'a>(
    obj: &'a AnimObject,
    data: &'a ObjectRenderData,
    depth: usize,
    out: &mut Vec<RenderItem<'a>>,
    object_lookup: &HashMap<Uuid, usize>,
    objects: &'a [(AnimObject, ObjectRenderData)],
) {
    let transform = obj.transform();

    let pipeline = data.pipeline;

    let base_depth = transform.z + depth as f32 * 0.001;

    out.push(RenderItem {
        object: obj,
        data,
        pipeline,
        depth: base_depth,
    });

    for child_uuid in &transform.children {
        let child = &objects[object_lookup[child_uuid]];
        // children always above parent
        collect_render_items(&child.0, &child.1, depth + 1, out, object_lookup, objects);
    }
}
fn build_buckets<'a>(
    object_lookup: &HashMap<Uuid, usize>,
    objects: &'a [(AnimObject, ObjectRenderData)],
) -> RenderBuckets<'a> {
    let mut buckets: HashMap<PipelineKind, Vec<RenderItem<'a>>> = HashMap::new();

    for (obj, data) in objects {
        let mut flat = Vec::new();

        collect_render_items(obj, data, 0, &mut flat, &object_lookup, objects);

        for item in flat {
            buckets.entry(item.pipeline).or_default().push(item);
        }
    }

    RenderBuckets { buckets }
}
pub fn draw_objects(
    render_pass: &mut wgpu::RenderPass,
    pipelines: &HashMap<PipelineKind, PipelineData>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: Scene,
    renderer: &mut Renderer,
) {
    if scene.objects.len() == 0 {
        debug!("No objects, skipping drawing buckets");
        return;
    }
    let buckets = build_buckets(scene.object_lookup, scene.objects);
    renderer.update_render_buffers(device, queue, &scene);
    renderer.draw_buckets(render_pass, pipelines, buckets);
}

#[derive(Clone, Copy)]
pub struct RenderingSettings {
    pub camera: Camera,
    pub background_color: Color,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
}
