pub mod anim_object;
pub mod anim_op;
mod anim_render;
pub mod animator;
pub mod audio_player;
pub mod conversion;
pub mod encoder;
pub mod nv12;
pub mod preview;
pub mod projection;
mod readback;
pub mod renderer;
pub mod sfx;
pub mod types;

use crate::anim_op::AnimOperation;
use crate::conversion::convert_anim_ops;
use crate::sfx::{AudioEngine, ScheduledSound};
use anyhow::{Context, Result, bail};
use log::{debug, info};

pub use scal_core::{self, Color, Ease, Time as CoreSeconds};

use std::path::PathBuf;

use scal_core::{CodecType, EncodingSettings, Project, RenderingSettings, Time, Sfx};
use tokio::runtime::Handle;

#[derive(serde::Deserialize, Clone)]
struct Config {
    animation: AnimationConfig,
    rendering: RenderingConfig,
    encoding: EncodingConfig,
}

#[derive(serde::Deserialize, Clone)]
struct AnimationConfig {
    #[serde(default = "default_animation_binary")]
    binary: String,
}

fn default_animation_binary() -> String {
    "cargo run --bin animation".to_string()
}

#[derive(serde::Deserialize, Clone)]
struct RenderingConfig {
    width: u32,
    height: u32,
    fps: u32,
    #[serde(default = "default_buffer_count")]
    buffer_count: u32,
    #[serde(default = "default_text_resolution_multiplier")]
    text_resolution_multiplier: f32,
}

fn default_buffer_count() -> u32 {
    3
}
fn default_text_resolution_multiplier() -> f32 {
    1.0
}

#[derive(serde::Deserialize, Clone)]
struct EncodingConfig {
    output_path: String,
    codec_type: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = colog::default_builder();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        bail!("Usage: scal <render|preview>");
    }

    let mode = &args[1];
    match mode.as_str() {
        "render" => run_render().await?,
        "preview" => run_preview().await?,
        _ => bail!("Unknown mode: {mode}. Use 'render' or 'preview'."),
    }

    Ok(())
}

async fn run_preview() -> Result<()> {
    let config_path = PathBuf::from("Config.toml");
    if !config_path.exists() {
        bail!("Config.toml not found in current directory");
    }
    let content = std::fs::read_to_string(&config_path).context("Failed to read Config.toml")?;
    let config: Config = toml::from_str(&content).context("Failed to parse Config.toml")?;

    preview::run_preview(config).await
}

async fn run_render() -> Result<()> {
    let config_path = PathBuf::from("Config.toml");
    if !config_path.exists() {
        bail!("Config.toml not found in current directory");
    }
    let content = std::fs::read_to_string(&config_path).context("Failed to read Config.toml")?;
    let config: Config = toml::from_str(&content).context("Failed to parse Config.toml")?;

    let codec_type = match config.encoding.codec_type.to_uppercase().as_str() {
        "H264" => scal_core::CodecType::H264,
        "H264NVENC" => scal_core::CodecType::H264Nvenc,
        "H264AMF" => scal_core::CodecType::H264Amf,
        "H264QSV" => scal_core::CodecType::H264Qsv,
        "H264VIDEOTOOLBOX" => scal_core::CodecType::H264Videotoolbox,
        "PRORES" => scal_core::CodecType::PRORES,
        other => bail!("Unknown codec type: {other}"),
    };

    let rendering = RenderingSettings {
        width: config.rendering.width,
        height: config.rendering.height,
        fps: config.rendering.fps,
        buffer_count: config.rendering.buffer_count,
        text_resolution_multiplier: config.rendering.text_resolution_multiplier,
    };

    let encoding = EncodingSettings {
        output_path: config.encoding.output_path,
        codec_type,
    };

    log::info!(
        "Scal Render - spawning animation binary: {}",
        config.animation.binary
    );

    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&config.animation.binary)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("Failed to spawn animation binary")?;

    let output = child
        .wait_with_output()
        .context("Failed to wait for animation binary")?;

    if !output.status.success() {
        bail!("Animation binary exited with error: {}", output.status);
    }

    let data = output.stdout;
    let encoded = data.as_slice();

    if encoded.len() < 8 {
        bail!("No data received from animation binary");
    }

    let (len_bytes, rest) = encoded.split_at(8);
    let len = usize::try_from(u64::from_le_bytes(len_bytes.try_into()?))?;

    if rest.len() < len {
        bail!("Incomplete project data received");
    }

    let project: scal_core::Project =
        bincode::deserialize(&rest[..len]).context("Failed to deserialize project")?;

    log::info!(
        "Received project with {} timeline operations",
        project.timeline.len()
    );

    let handle = Handle::current();
    render_project(&handle, encoding, rendering, project)
        .await
        .context("Failed to render project")?;

    log::info!("Render complete!");

    Ok(())
}

const BYTES_PER_PIXEL: u32 = 4; //RGBA
async fn run_loop(
    tokio_handle: &tokio::runtime::Handle,
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    project: Project,
    mut animations: Vec<AnimOperation>,
) -> Result<()> {
    fn op_end_time(
        op: &AnimOperation,
        start_time: Time,
        out: &mut Vec<(Sfx, Time, Option<scal_core::SourceLoc>)>,
    ) -> Time {
        match op {
            AnimOperation::PlaySound(sfx, video_delay, source_loc) => {
                let abs_time = start_time + video_delay;
                debug!(
                    "audio: {} at abs_time={}, seek={}",
                    sfx.path, abs_time, sfx.time_offset
                );
                out.push((sfx.clone(), abs_time, source_loc.clone()));
                start_time
            }
            AnimOperation::All(children, _) => {
                let mut max_end = start_time;
                for child in children {
                    let end = op_end_time(child, start_time, out);
                    if end > max_end {
                        max_end = end;
                    }
                }
                max_end
            }
            AnimOperation::Sequence(children, _) => {
                let mut t = start_time;
                for child in children {
                    t = op_end_time(child, t, out);
                }
                t
            }
            AnimOperation::Wait(dur, _)
            | AnimOperation::CodeAddLines(_, _, _, dur, _, _, _)
            | AnimOperation::CodeModifyLine(_, _, _, dur, _, _, _)
            | AnimOperation::CodeRemoveLines(_, _, dur, _, _, _)
            | AnimOperation::TransformMovePos(_, _, dur, _, _)
            | AnimOperation::TransformMoveToObj(_, _, _, dur, _, _)
            | AnimOperation::TransformRotate(_, _, dur, _, _)
            | AnimOperation::TransformScale(_, _, dur, _, _)
            | AnimOperation::TerminalTypeInput(_, _, _, _, _, dur, _, _, _)
            | AnimOperation::TerminalOutput(_, _, dur, _, _, _)
            | AnimOperation::ObjectColor(_, _, dur, _, _) => start_time + dur,
            AnimOperation::CodeHighlight(_, action, _) => {
                start_time + action.duration_and_curve().0
            }
            AnimOperation::Instantiate(..) => start_time,
        }
    }

    let mut sfx_sounds: Vec<(Sfx, Time, Option<scal_core::SourceLoc>)> = vec![];
    let mut time = 0.0;
    for op in &animations {
        time = op_end_time(op, time, &mut sfx_sounds);
    }
    debug!(
        "collect_sounds total: {}, total_dur={}",
        sfx_sounds.len(),
        time
    );

    let scheduled: Vec<ScheduledSound> = sfx_sounds
        .into_iter()
        .map(|(s, abs_start_time, source_loc_opt)| {
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
                source_loc: source_loc_opt,
            };
            debug!(
                "ScheduledSound: path={}, start_time={}, seek={}, duration={}, pitch={}",
                ss.path, ss.start_time, ss.seek_offset, ss.duration, ss.pitch
            );
            ss
        })
        .collect();
    let audio_engine = if scheduled.is_empty() {
        None
    } else {
        Some(AudioEngine::new(scheduled))
    };

    info!("Starting rendering loop...");
    if !(rendering_settings.width * 4).is_multiple_of(256) {
        bail!("Wgpu needs the bytes_per_row(width * 4) value to be multiple of 256");
    }
    let codec_type = encoding_settings.codec_type;
    let use_nv12 = matches!(
        codec_type,
        CodecType::H264
            | CodecType::H264Nvenc
            | CodecType::H264Amf
            | CodecType::H264Qsv
            | CodecType::H264Videotoolbox
    );
    let pixel_buffer_size = if use_nv12 {
        (rendering_settings.width * rendering_settings.height * 3 / 2) as usize
    } else {
        (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize
    };
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .context("failed to request wgpu adapter")?;

    info!(
        "Adapter: {:?} (backend: {:?})",
        adapter.get_info().name,
        adapter.get_info().backend
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .context("failed to request wgpu device")?;
    readback::init_buffers(rendering_settings.buffer_count, pixel_buffer_size, &device)
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
        project.scene_settings,
        codec_type,
    )
    .await
    .context("while rendering the animation")?;

    Ok(())
}

pub async fn render_project(
    tokio_handle: &tokio::runtime::Handle,
    core_encoding: scal_core::EncodingSettings,
    core_rendering: scal_core::RenderingSettings,
    project: scal_core::Project,
) -> Result<()> {
    let encoding = EncodingSettings {
        output_path: core_encoding.output_path,
        codec_type: core_encoding.codec_type,
    };

    let rendering = RenderingSettings {
        width: core_rendering.width,
        height: core_rendering.height,
        fps: core_rendering.fps,
        buffer_count: core_rendering.buffer_count,
        text_resolution_multiplier: core_rendering.text_resolution_multiplier,
    };

    let default_theme = project.scene_settings.default_theme.clone();
    let animations = convert_anim_ops(project.timeline.clone(), &default_theme)?;

    run_loop(tokio_handle, encoding, rendering, project, animations).await
}
