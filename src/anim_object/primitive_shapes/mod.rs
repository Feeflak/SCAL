pub mod mesh;

use crate::anim_object::render::PipelineData;
use crate::renderer::Vertex;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct Square {
    pub size: Vec2,
    /// TODO: Implement better square mesh generation for this to work
    pub corner_radius: f32,
    pub color: Color,
}
pub fn create_shape_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
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
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        immediate_size: 0,
        label: Some("shape_pipeline_layout"),
        bind_group_layouts: &[],
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
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });
    PipelineData {
        pipeline,
        bind_groups: vec![],
    }
}
