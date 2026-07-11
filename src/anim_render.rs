use crate::{
    anim_object::{render::PipelineKind, text::render::TextRenderer},
    animator::Animator,
    encoder::CodecType,
    types::Seconds,
};
use anyhow::{Context, Ok, Result};
use tokio::sync::mpsc::Sender;

use log::info;
use wgpu::{Device, Texture};

use crate::{
    BYTES_PER_PIXEL,
    anim_op::AnimOperation,
    encoder::{self, EncoderComunication},
    nv12::Nv12Converter,
    readback::{self, ReadbackRing},
    renderer::{Renderer, RenderingSettings},
};

#[derive(Debug)]
pub struct AnimationState {
    pub anim_op: AnimOperation,
    pub storage: Vec<f32>,
    pub time: Seconds,
}

impl AnimationState {
    pub fn new(anim: AnimOperation) -> Result<Self> {
        Ok(Self {
            storage: vec![],
            anim_op: anim
                .try_into()
                .context("couldn't convert anim_op to animation")?,
            time: 0.0,
        })
    }
}
pub async fn render_animations(
    queue: wgpu::Queue,
    animations: Vec<AnimOperation>,
    mut readback_ring: ReadbackRing,
    encoder_send: Sender<encoder::EncoderComunication>,
    device: Device,
    rendering_settings: RenderingSettings,
    codec_type: CodecType,
) -> Result<()> {
    let mut renderer = Renderer::new(&device);
    const MSAA_SAMPLE_COUNT: u32 = 4;

    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa render target"),
        size: wgpu::Extent3d {
            width: rendering_settings.width,
            height: rendering_settings.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let resolve_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("resolve texture"),
        size: wgpu::Extent3d {
            width: rendering_settings.width,
            height: rendering_settings.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let use_nv12 = matches!(codec_type, CodecType::H264 | CodecType::H264Nvenc);
    let nv12 = if use_nv12 {
        Some(Nv12Converter::new(
            &device,
            &resolve_texture,
            rendering_settings.width,
            rendering_settings.height,
        ))
    } else {
        None
    };

    let (pipelines, text_renderer) = {
        let mut pipelines = crate::anim_object::render::get_pipelines(&device, MSAA_SAMPLE_COUNT);

        let text_renderer =
            TextRenderer::new(&device, rendering_settings.text_resolution_multiplier);
        pipelines
            .get_mut(&PipelineKind::Text)
            .expect("there was no text pipeline")
            .bind_groups
            .push(text_renderer.bind_group.clone());
        (pipelines, text_renderer)
    };
    let mut animator = Animator::new(
        animations,
        rendering_settings.fps,
        rendering_settings.camera,
        rendering_settings.text_resolution_multiplier,
    )
    .context("while initiating the animator")?;

    let mut timing_animation = std::time::Duration::ZERO;
    let mut timing_render = std::time::Duration::ZERO;
    let mut timing_wait_slot = std::time::Duration::ZERO;
    let mut timing_gpu_copy = std::time::Duration::ZERO;
    let mut timing_submit = std::time::Duration::ZERO;
    let mut frame_count = 0u64;
    let timing_start = std::time::Instant::now();

    loop {
        let t0 = std::time::Instant::now();
        let frame_animation_data = animator
            .animate_next_frame()
            .with_context(|| format!("while rendering frame {}", frame_count + 1))?;
        timing_animation += t0.elapsed();

        let frame_animation_data = match frame_animation_data {
            Some(d) => d,
            None => break,
        };

        let scene = frame_animation_data.scene;

        let msaa_view = msaa_texture.create_view(&Default::default());
        let resolve_view = resolve_texture.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });

        let t1 = std::time::Instant::now();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(rendering_settings.background_color.into()),
                        store: wgpu::StoreOp::Discard,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(glyph_data) = frame_animation_data.glyph_update_data {
                text_renderer.update_glyphs_if_needed(glyph_data, &queue);
            }

            crate::renderer::draw_scene(
                &mut render_pass,
                &pipelines,
                &device,
                &queue,
                scene,
                &mut renderer,
            )
            .context("while drawing scene")?;
        }
        timing_render += t1.elapsed();

        let t2a = std::time::Instant::now();
        let slot = readback_ring
            .next()
            .await
            .context("renderer channel was closed")?;
        timing_wait_slot += t2a.elapsed();

        let t2b = std::time::Instant::now();
        if use_nv12 {
            let t_sub = std::time::Instant::now();
            queue.submit(Some(encoder.finish()));
            timing_submit += t_sub.elapsed();

            nv12.as_ref()
                .context("NV12 converter not initialized but codec requires NV12")?
                .run_and_copy(&device, &queue, slot, encoder_send.clone());
        } else {
            copy_texture_to_buffer(
                encoder_send.clone(),
                &queue,
                rendering_settings,
                &device,
                &resolve_texture,
                slot,
            )
            .context("while copying texture to the buffer")?;

            let t2c = std::time::Instant::now();
            queue.submit(Some(encoder.finish()));
            timing_submit += t2c.elapsed();
        }
        timing_gpu_copy += t2b.elapsed();

        frame_count += 1;
    }

    let total =
        timing_animation + timing_render + timing_wait_slot + timing_gpu_copy + timing_submit;
    info!("=== Pipeline Timing ===");
    info!(
        "Total frames: {frame_count}  |  Wall time: {:.2?}",
        timing_start.elapsed()
    );
    info!(
        "Animation       | total: {:.3}s  | avg: {:.1}ms  | {:.0}%",
        timing_animation.as_secs_f64(),
        timing_animation.as_secs_f64() / frame_count as f64 * 1000.0,
        timing_animation.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    info!(
        "GPU Render (CPU)| total: {:.3}s  | avg: {:.1}ms  | {:.0}%",
        timing_render.as_secs_f64(),
        timing_render.as_secs_f64() / frame_count as f64 * 1000.0,
        timing_render.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    info!(
        "Wait Slot       | total: {:.3}s  | avg: {:.1}ms  | {:.0}%",
        timing_wait_slot.as_secs_f64(),
        timing_wait_slot.as_secs_f64() / frame_count as f64 * 1000.0,
        timing_wait_slot.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    info!(
        "GPU Copy+Poll   | total: {:.3}s  | avg: {:.1}ms  | {:.0}%",
        timing_gpu_copy.as_secs_f64(),
        timing_gpu_copy.as_secs_f64() / frame_count as f64 * 1000.0,
        timing_gpu_copy.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    info!(
        "Submit Render   | total: {:.3}s  | avg: {:.1}ms  | {:.0}%",
        timing_submit.as_secs_f64(),
        timing_submit.as_secs_f64() / frame_count as f64 * 1000.0,
        timing_submit.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    info!("Finished Rendering");
    encoder_send.send(EncoderComunication::Finish).await?;

    info!("Waiting for the encoder to finish");
    // Wait until encoder finishes to avoid any issues
    encoder_send.closed().await;
    Ok(())
}

fn copy_texture_to_buffer(
    encoder_send: Sender<encoder::EncoderComunication>,
    queue: &wgpu::Queue,
    settings: RenderingSettings,
    device: &Device,
    texture: &Texture,
    slot: &readback::Slot,
) -> Result<()> {
    let id = slot.id;
    // debug!("MAP: {id}");

    let mut cmd = device.create_command_encoder(&Default::default());

    cmd.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &slot.buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(settings.width * BYTES_PER_PIXEL),
                rows_per_image: Some(settings.height),
            },
        },
        wgpu::Extent3d {
            width: settings.width,
            height: settings.height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(cmd.finish()));

    let buffer = slot.buffer.clone();

    slot.buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            if result.is_err() {
                return;
            }

            let data = buffer.slice(..).get_mapped_range();
            let owned = data.to_vec();

            drop(data);
            // debug!("UNMAP: {id}");
            buffer.unmap();

            encoder_send
                .try_send(EncoderComunication::FrameData { id, bytes: owned })
                .ok();
        });
    device.poll(wgpu::PollType::wait_indefinitely())?;
    Ok(())
}
