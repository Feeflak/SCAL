use anyhow::{Context, Result};
pub(crate) fn draw_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    frame: u32,
) {
    let view = texture.create_view(&Default::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (frame as f64 / 300.0).sin() * 0.5 + 0.5,
                        g: (frame as f64 / 300.0),
                        b: 0.8,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],

            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // draw your triangles here
    }

    queue.submit(Some(encoder.finish()));
}

#[derive(Clone, Copy)]
pub struct RenderingSettings {
    pub buffer_count: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}
