pub mod mesh;

use glam::{Vec2, vec2};

use crate::anim_object::Transform;
use crate::anim_object::object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult};
use crate::anim_object::primitive_shapes::mesh::{
    generate_circle_mesh_data, generate_polygon_mesh_data, generate_rectangle_mesh_data,
};
use crate::anim_object::render::PipelineData;
use crate::renderer::{Vertex, camera_bind_group_layout, transform_bind_group_layout};
use crate::types::*;

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub size: Vec2,
    pub corner_radius: f32,
    pub color: Color,
    pub transform: Transform,
}

impl AnimObjectTrait for Rectangle {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        self.size
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        generate_rectangle_mesh_data(self)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Circle {
    pub radius: f32,
    pub color: Color,
    pub transform: Transform,
}

impl AnimObjectTrait for Circle {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        let d = self.radius * 2.0;
        vec2(d, d)
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        generate_circle_mesh_data(self)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Polygon {
    pub radius: f32,
    pub sides: u32,
    pub color: Color,
    pub transform: Transform,
}

impl AnimObjectTrait for Polygon {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        let d = self.radius * 2.0;
        vec2(d, d)
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        generate_polygon_mesh_data(self)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
pub fn create_shape_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    sample_count: u32,
) -> PipelineData {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shape_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shape.wgsl").into()),
    });

    let vertex_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        immediate_size: 0,
        label: Some("shape_pipeline_layout"),
        bind_group_layouts: &[
            Some(&camera_bind_group_layout(device)),
            Some(&transform_bind_group_layout(device)),
        ],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        multiview_mask: None,
        cache: None,
        label: Some("shape_pipeline"),
        layout: Some(&pipeline_layout),

        vertex: wgpu::VertexState {
            module: &shader,
            compilation_options: Default::default(),
            entry_point: Some("vs_main"),
            buffers: &[vertex_layout],
        },

        fragment: Some(wgpu::FragmentState {
            compilation_options: Default::default(),
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },

        depth_stencil: None,

        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });
    PipelineData {
        pipeline,
        bind_groups: vec![],
    }
}
