use crate::{
    encoder::{Encoder, EncoderComunication, EncodingSettings},
    readback::{ReadbackRing, SLOTS, Slot},
    renderer::RenderingSettings,
};
use anyhow::{Context, Result, bail};
use log::{info, trace};
use tokio::sync::mpsc::Sender;
use wgpu::*;

pub mod encoder;
mod readback;
pub mod renderer;

const BYTES_PER_PIXEL: u32 = 4; //RGBA
pub async fn run_loop(
    tokio_handle: &tokio::runtime::Handle,
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    frames_to_render: u32,
) -> Result<()> {
    info!("Starting rendering loop...");
    if (rendering_settings.width * 4) % 256 != 0 {
        bail!("Wgpu needs the bytes_per_row(width * 4) value to be multiple of 256");
    }
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    readback::init_buffers(
        rendering_settings.buffer_count,
        (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize,
        &device,
    )
    .context("while initializing buffers")?;
    let (renderer_send, renderer_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    for i in 0..rendering_settings.buffer_count as usize {
        renderer_send.send(i).await.unwrap();
    }
    let (encoder_send, encoder_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    let mut ring = readback::ReadbackRing::new(renderer_rec);
    encoder::start_encoding_task(
        encoding_settings,
        tokio_handle,
        rendering_settings,
        encoder_rec,
        renderer_send,
    )
    .context("while initializing the encoder")?;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render target"),
        size: wgpu::Extent3d {
            width: rendering_settings.width,
            height: rendering_settings.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    for frame in 0..frames_to_render {
        info!("Frame: {frame}/{frames_to_render}");
        trace!("draw_frame");
        renderer::draw_frame(&device, &queue, &texture, frame);

        let slot = ring.next().await.context("renderer channel was closed")?;
        trace!("copy_texture_to_buffer");
        copy_texture_to_buffer(
            slot,
            &texture,
            &queue,
            &rendering_settings,
            &device,
            encoder_send.clone(),
        )
        .context("while copying texture to the buffer")?
    }
    info!("Finished Rendering");
    encoder_send.send(EncoderComunication::Finish).await?;

    info!("Waiting for the encoder to finish");
    // Wait until encoder finishes to avoid any issues
    encoder_send.closed().await;
    Ok(())
}
fn copy_texture_to_buffer(
    slot: &Slot,
    texture: &Texture,
    queue: &Queue,
    rendering_settings: &RenderingSettings,
    device: &wgpu::Device,
    encoder_send: Sender<EncoderComunication>,
) -> Result<()> {
    let id = slot.id;
    trace!("MAP: {id}");

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

    let buffer = slot.buffer.clone();

    slot.buffer
        .slice(..)
        .map_async(MapMode::Read, move |result| {
            if result.is_err() {
                return;
            }

            let data = buffer.slice(..).get_mapped_range();
            let owned = data.to_vec();

            drop(data);
            trace!("UNMAP: {id}");
            buffer.unmap();

            encoder_send
                .try_send(EncoderComunication::FrameData { id, bytes: owned })
                .ok();
        });
    device.poll(wgpu::PollType::wait_indefinitely())?;
    Ok(())
}
