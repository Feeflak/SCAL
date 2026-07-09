use crate::encoder;
use wgpu::util::DeviceExt;

pub struct Nv12Converter {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    storage_buffer: wgpu::Buffer,
    _params_buffer: wgpu::Buffer,
    nv12_byte_size: u64,
    dispatch_x: u32,
    dispatch_y: u32,
}

impl Nv12Converter {
    pub fn new(
        device: &wgpu::Device,
        resolve_texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Self {
        let resolve_view = resolve_texture.create_view(&Default::default());
        assert!(
            width % 4 == 0,
            "NV12 converter requires width to be a multiple of 4"
        );
        let nv12_byte_size = (width as u64 * height as u64 * 3 / 2) as u64;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rgba_to_nv12"),
            source: wgpu::ShaderSource::Wgsl(include_str!("rgba_to_nv12.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nv12 bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nv12 pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rgba to nv12"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nv12 storage buffer"),
            size: nv12_byte_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let y_stride = (width + 3) / 4;
        let params = [width, height, y_stride, 0u32];
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nv12 params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nv12 bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&resolve_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let dx = (width + 3) / 4;
        let dy = (height + 1) / 2;

        Self {
            pipeline,
            bind_group,
            storage_buffer,
            _params_buffer: params_buffer,
            nv12_byte_size,
            dispatch_x: dx,
            dispatch_y: dy,
        }
    }

    pub(crate) fn run_and_copy(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: &crate::readback::Slot,
        encoder_send: tokio::sync::mpsc::Sender<encoder::EncoderComunication>,
    ) {
        let wg_x = (self.dispatch_x + 7) / 8;
        let wg_y = (self.dispatch_y + 7) / 8;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nv12 compute"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("rgba to nv12"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        encoder.copy_buffer_to_buffer(
            &self.storage_buffer,
            0,
            &slot.buffer,
            0,
            self.nv12_byte_size,
        );

        queue.submit(Some(encoder.finish()));

        let buffer = slot.buffer.clone();
        let id = slot.id;
        let size = self.nv12_byte_size as usize;

        slot.buffer
            .slice(..self.nv12_byte_size)
            .map_async(wgpu::MapMode::Read, move |result| {
                if result.is_err() {
                    return;
                }
                let data = buffer.slice(..).get_mapped_range();
                let owned = data[..size].to_vec();
                drop(data);
                buffer.unmap();
                encoder_send
                    .try_send(encoder::EncoderComunication::FrameData { id, bytes: owned })
                    .ok();
            });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    }
}
