pub mod mesh;

use glam::Vec2;

use crate::anim_object::Transform;
use crate::anim_object::image::mesh::generate_image_mesh_data;
use crate::anim_object::object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult};
use crate::anim_object::render::PipelineData;
use crate::renderer::{Vertex, camera_bind_group_layout, transform_bind_group_layout};
use crate::types::*;

#[derive(Clone, Debug)]
pub enum StretchMode {
    Fill,
    Fit,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub path: String,
    pub size: Vec2,
    pub color: Color,
    pub stretch: StretchMode,
    pub transform: Transform,
}

impl AnimObjectTrait for Image {
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
        generate_image_mesh_data(self)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        let path = self.path.clone();
        Some(Box::new(
            move |device, queue| match crate::anim_object::image::load_image_bind_group(
                device, queue, &path,
            ) {
                Ok(bg) => vec![bg],
                Err(e) => {
                    log::error!("Failed to load image bind group: {:?}", e);
                    vec![]
                }
            },
        ))
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

pub fn create_image_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    sample_count: u32,
) -> PipelineData {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("image_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("image.wgsl").into()),
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

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("image_texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        immediate_size: 0,
        label: Some("image_pipeline_layout"),
        bind_group_layouts: &[
            Some(&camera_bind_group_layout(device)),
            Some(&transform_bind_group_layout(device)),
            Some(&texture_bind_group_layout),
        ],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        multiview_mask: None,
        cache: None,
        label: Some("image_pipeline"),
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

pub fn load_image_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> Result<wgpu::BindGroup, anyhow::Error> {
    let img = image::ImageReader::open(path)?.decode()?.to_rgba8();
    let (width, height) = img.dimensions();
    let pixels = img.into_raw();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("image_tex_{path}")),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    let texture_view = texture.create_view(&Default::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("image_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("image_loader_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("image_bg_{path}")),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok(bind_group)
}
