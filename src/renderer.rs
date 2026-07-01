use log::{debug, info};
use std::collections::HashMap;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use crate::anim_object::AnimObject;
use crate::anim_object::render::{ObjectRenderData, PipelineData, PipelineKind};
use crate::animator::Scene;
use crate::types::{Color, Vec2};

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
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: usize,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_size: usize,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object Vertex Buffer"),
            contents: &vec![],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object Index Buffer"),
            contents: &vec![],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            vertex_buffer_size: 0,
            index_buffer,
            index_buffer_size: 0,
        }
    }
    pub fn update_text_glyphs() {}

    pub fn update_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
        indices: &[Index],
    ) {
        if self.index_buffer_size != indices.len() {
            self.index_buffer_size = indices.len();
            self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
        }

        if self.vertex_buffer_size != vertices.len() {
            self.vertex_buffer_size = vertices.len();
            self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        }
        info!("update_mesh- vertices: {vertices:?}, indices: {indices:?}");
    }

    pub fn draw_buckets(
        &self,
        render_pass: &mut wgpu::RenderPass,
        pipelines: &HashMap<PipelineKind, PipelineData>,
        buckets: RenderBuckets,
    ) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for (pipeline_kind, mut items) in buckets.buckets {
            items.sort_by(|a, b| a.depth.total_cmp(&b.depth));

            let pipeline_data = &pipelines[&pipeline_kind];
            render_pass.set_pipeline(&pipeline_data.pipeline);

            for (i, bind_group) in pipeline_data.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }
            for item in items {
                info!(
                    "item.data.indices_base_index- {}",
                    item.data.indices_base_index
                );
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
    renderer.update_mesh(device, queue, scene.vertices, scene.indices);
    renderer.draw_buckets(render_pass, pipelines, buckets);
}

#[derive(Clone, Copy)]
pub struct RenderingSettings {
    pub background_color: Color,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
}
