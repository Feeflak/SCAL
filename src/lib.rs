use crate::anim_op::AnimOP;
use crate::encoder::EncodingSettings;
use crate::renderer::RenderingSettings;
use anyhow::{Context, Result, bail};
use log::info;

pub mod anim_object;
pub mod anim_op;
mod anim_render;
pub mod animator;
pub mod encoder;
pub mod projection;
mod readback;
pub mod renderer;
pub mod types;

const BYTES_PER_PIXEL: u32 = 4; //RGBA
pub async fn run_loop(
    tokio_handle: &tokio::runtime::Handle,
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    mut animations: Vec<AnimOP>,
) -> Result<()> {
    animations.reverse();
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
    encoder::start_encoding_task(
        encoding_settings,
        tokio_handle,
        rendering_settings,
        encoder_rec,
        renderer_send,
    )
    .context("while initializing the encoder")?;
    anim_render::render_animations(
        queue,
        animations,
        readback::ReadbackRing::new(renderer_rec),
        encoder_send,
        device,
        rendering_settings,
    )
    .await
    .context("while rendering the animation")?;

    Ok(())
}
