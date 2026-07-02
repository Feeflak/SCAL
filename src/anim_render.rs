use crate::{
    BYTES_PER_PIXEL,
    anim_object::{self, AnimObject, render::PipelineKind, text::render::TextRenderer},
    animator::Animator,
    types::Seconds,
};
use anyhow::{Context, Ok, Result};
use tokio::sync::mpsc::Sender;

use log::{debug, info};
use wgpu::{Device, Texture};

use crate::{
    anim_op::AnimOP,
    encoder::{self, EncoderComunication},
    readback::{self, ReadbackRing},
    renderer::{Renderer, RenderingSettings},
};

#[derive(Debug)]
pub struct AnimationState {
    pub anim_op: AnimOP,
    pub storage: Vec<f32>,
    pub time: Seconds,
}

impl AnimationState {
    pub fn new(anim: AnimOP) -> Result<Self> {
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
    animations: Vec<AnimOP>,
    mut readback_ring: ReadbackRing,
    encoder_send: Sender<encoder::EncoderComunication>,
    device: Device,
    rendering_settings: RenderingSettings,
) -> Result<()> {
    let mut renderer = Renderer::new(&device);
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

    let (pipelines, mut text_renderer) = {
        let mut pipelines = crate::anim_object::render::get_pipelines(&device);

        let text_renderer = TextRenderer::new(&device);
        pipelines
            .get_mut(&PipelineKind::Text)
            .expect("there was no text pipeline")
            .bind_groups
            .push(text_renderer.bind_group.clone());
        (pipelines, text_renderer)
    };
    let mut animator = Animator::new(animations, rendering_settings.fps,rendering_settings.camera)
        .context("while initiating the animator")?;
    while let Some(frame_animation_data) = animator
        .animate_next_frame()
        .context("while rendering next frame")?
    {
        let scene = frame_animation_data.scene;

        let texture_view = texture.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(rendering_settings.background_color.into()),
                        store: wgpu::StoreOp::Store,
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

            crate::renderer::draw_objects(
                &mut render_pass,
                &pipelines,
                &device,
                &queue,
                scene,
                &mut renderer,
            );
        }

        let slot = readback_ring
            .next()
            .await
            .context("renderer channel was closed")?;

        copy_texture_to_buffer(
            encoder_send.clone(),
            &queue,
            rendering_settings,
            &device,
            &texture,
            slot,
        )
        .context("while copying texture to the buffer")?;

        queue.submit(Some(encoder.finish()));
    }

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
    debug!("MAP: {id}");

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
            debug!("UNMAP: {id}");
            buffer.unmap();

            encoder_send
                .try_send(EncoderComunication::FrameData { id, bytes: owned })
                .ok();
        });
    device.poll(wgpu::PollType::wait_indefinitely())?;
    Ok(())
}
