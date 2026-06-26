use crate::{
    encoder::{Encoder, EncodingSettings},
    readback::{ReadbackRing, Slot},
    renderer::RenderingSettings,
};
use anyhow::{Context, Result};
use tokio::sync::mpsc::Sender;
use wgpu::*;

mod encoder;
mod readback;
mod renderer;

const BYTES_PER_PIXEL: u32 = 4; //RGBA
pub async fn run_loop(
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    texture: Texture,
    device: wgpu::Device,
    frames: u32,
) -> Result<()> {
    readback::init_buffers(
        rendering_settings.buffer_count,
        (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize,
        &device,
    );
    let (renderer_send, renderer_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    let (encoder_send, encoder_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    let mut ring = readback::ReadbackRing::new(renderer_rec);

    let mut encoder = Encoder::new(
        encoding_settings,
        rendering_settings.width,
        rendering_settings.height,
        rendering_settings.fps,
    )?;

    texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render target"),
        size: wgpu::Extent3d {
            width: rendering_settings.width,
            height: rendering_settings.width,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    for frame in 0..frames {
        renderer::draw_frame(&device, &queue, &texture, frame);

        let slot = ring.next().await.context("renderer channel was closed")?;
        copy_texture_to_buffer(
            slot,
            &texture,
            queue,
            &rendering_settings,
            &device,
            encoder_send,
            renderer_send,
        )
    }

    Ok(())
}
fn copy_texture_to_buffer(
    slot: &Slot,
    texture: &Texture,
    queue: &Queue,
    rendering_settings: &RenderingSettings,
    device: &wgpu::Device,
    encoder_send: Sender<usize>,
    renderer_send: Sender<usize>,
) {
    let mut cmd = device.create_command_encoder(&Default::default());

    cmd.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &slot.buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(rendering_settings.width * BYTES_PER_PIXEL),
                rows_per_image: Some(rendering_settings.height),
            },
        },
        wgpu::Extent3d {
            width: rendering_settings.width,
            height: rendering_settings.height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(cmd.finish()));
    let slice = slot.buffer.slice(..);
    slice.map_async(MapMode::Read, {
        let id = slot.id;
        move |result| {
            result.expect("async map the buffer ");
            renderer_send
                .blocking_send(id)
                .expect("renderer channel was closed");
            encoder_send
                .blocking_send(id)
                .expect("encoder channel was closed");
        }
    });
}
