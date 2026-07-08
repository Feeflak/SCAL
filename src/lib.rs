use crate::anim_op::AnimOP;
use crate::encoder::{CodecType, EncodingSettings};
use crate::renderer::RenderingSettings;
use crate::sfx::{AudioEngine, ScheduledSound};
use crate::types::{Seconds, Sfx};
use anyhow::{Context, Result, bail};
use log::{debug, info};

pub mod anim_object;
pub mod anim_op;
mod anim_render;
pub mod animator;
pub mod encoder;
pub mod nv12;
pub mod prelude;
pub mod projection;
mod readback;
pub mod renderer;
pub mod sfx;
pub mod types;

const BYTES_PER_PIXEL: u32 = 4; //RGBA
pub async fn run_loop(
    tokio_handle: &tokio::runtime::Handle,
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    mut animations: Vec<AnimOP>,
) -> Result<()> {
    fn op_end_time(op: &AnimOP, start_time: Seconds, out: &mut Vec<(Sfx, Seconds)>) -> Seconds {
        match op {
            AnimOP::PlaySound(sfx, video_delay) => {
                let abs_time = start_time + video_delay;
                debug!("audio: {} at abs_time={}, seek={}", sfx.path, abs_time, sfx.time_offset);
                out.push((sfx.clone(), abs_time));
                start_time
            }
            AnimOP::All(children) => {
                let mut max_end = start_time;
                for child in children {
                    let end = op_end_time(child, start_time, out);
                    if end > max_end {
                        max_end = end;
                    }
                }
                max_end
            }
            AnimOP::Sequence(children) => {
                let mut t = start_time;
                for child in children {
                    t = op_end_time(child, t, out);
                }
                t
            }
            AnimOP::Wait(dur) => start_time + dur,
            AnimOP::CodeAddLines(_, _, _, dur, _, _)
            | AnimOP::CodeModifyLine(_, _, _, dur, _, _)
            | AnimOP::CodeRemoveLines(_, _, dur, _, _) => start_time + dur,
            AnimOP::CodeHighlight(_, action) => start_time + action.duration_and_curve().0,
            AnimOP::TransformMovePos(_, _, dur, _)
            | AnimOP::TransformMoveToObj(_, _, _, dur, _)
            | AnimOP::TransformRotate(_, _, dur, _)
            | AnimOP::TransformScale(_, _, dur, _) => start_time + dur,
            AnimOP::Instantiate(_) | AnimOP::Current { .. } => start_time,
        }
    }

    let mut sfx_sounds: Vec<(Sfx, Seconds)> = vec![];
    let mut time = 0.0;
    for op in &animations {
        time = op_end_time(op, time, &mut sfx_sounds);
    }
    debug!("collect_sounds total: {}, total_dur={}", sfx_sounds.len(), time);

    let scheduled: Vec<ScheduledSound> = sfx_sounds
        .into_iter()
        .map(|(s, abs_start_time)| {
            let pitch_var = if s.pitch_variation > 0.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let variation = rng.gen_range(-s.pitch_variation..s.pitch_variation);
                s.pitch * (1.0 + variation)
            } else {
                s.pitch
            };
            let ss = ScheduledSound {
                path: s.path,
                volume: s.volume,
                pitch: pitch_var,
                start_time: abs_start_time,
                seek_offset: s.time_offset,
                duration: s.duration,
            };
            debug!("ScheduledSound: path={}, start_time={}, seek={}, duration={}, pitch={}", ss.path, ss.start_time, ss.seek_offset, ss.duration, ss.pitch);
            ss
        })
        .collect();
    let audio_engine = if scheduled.is_empty() {
        None
    } else {
        Some(AudioEngine::new(scheduled))
    };

    animations.reverse();
    info!("Starting rendering loop...");
    if (rendering_settings.width * 4) % 256 != 0 {
        bail!("Wgpu needs the bytes_per_row(width * 4) value to be multiple of 256");
    }
    let codec_type = encoding_settings.codec_type;
    let use_nv12 = matches!(codec_type, CodecType::H264 | CodecType::H264Nvenc);
    let pixel_buffer_size = if use_nv12 {
        (rendering_settings.width * rendering_settings.height * 3 / 2) as usize
    } else {
        (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize
    };
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
        pixel_buffer_size,
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
        audio_engine,
    )
    .context("while initializing the encoder")?;
    anim_render::render_animations(
        queue,
        animations,
        readback::ReadbackRing::new(renderer_rec),
        encoder_send,
        device,
        rendering_settings,
        codec_type,
    )
    .await
    .context("while rendering the animation")?;

    Ok(())
}
