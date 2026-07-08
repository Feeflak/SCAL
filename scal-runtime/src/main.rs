use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use scal_core::{EncodingSettings, RenderingSettings};
use tokio::runtime::Handle;

#[derive(serde::Deserialize)]
struct Config {
    animation: AnimationConfig,
    rendering: RenderingConfig,
    encoding: EncodingConfig,
}

#[derive(serde::Deserialize)]
struct AnimationConfig {
    #[serde(default = "default_animation_binary")]
    binary: String,
}

fn default_animation_binary() -> String {
    "cargo run --bin animation".to_string()
}

#[derive(serde::Deserialize)]
struct RenderingConfig {
    width: u32,
    height: u32,
    fps: u32,
    #[serde(default = "default_buffer_count")]
    buffer_count: u32,
    #[serde(default = "default_text_resolution_multiplier")]
    text_resolution_multiplier: f32,
}

fn default_buffer_count() -> u32 { 3 }
fn default_text_resolution_multiplier() -> f32 { 1.0 }

#[derive(serde::Deserialize)]
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
        "preview" => bail!("Preview mode not yet implemented"),
        _ => bail!("Unknown mode: {}. Use 'render' or 'preview'.", mode),
    }

    Ok(())
}

async fn run_render() -> Result<()> {
    let config_path = PathBuf::from("Config.toml");
    if !config_path.exists() {
        bail!("Config.toml not found in current directory");
    }
    let content = std::fs::read_to_string(&config_path)
        .context("Failed to read Config.toml")?;
    let config: Config = toml::from_str(&content)
        .context("Failed to parse Config.toml")?;

    let codec_type = match config.encoding.codec_type.to_uppercase().as_str() {
        "H264" => scal_core::CodecType::H264,
        "H264NVENC" => scal_core::CodecType::H264Nvenc,
        "PRORES" => scal_core::CodecType::PRORES,
        other => bail!("Unknown codec type: {}", other),
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

    log::info!("Scal Render - spawning animation binary: {}", config.animation.binary);

    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&config.animation.binary)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("Failed to spawn animation binary")?;

    let output = child.wait_with_output()
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
    let len = u64::from_le_bytes(len_bytes.try_into()?) as usize;

    if rest.len() < len {
        bail!("Incomplete project data received");
    }

    let project: scal_core::Project = bincode::deserialize(&rest[..len])
        .context("Failed to deserialize project")?;

    log::info!(
        "Received project with {} timeline operations",
        project.timeline.len()
    );

    let handle = Handle::current();
    scal::render_project(&handle, encoding, rendering, project)
        .await
        .context("Failed to render project")?;

    log::info!("Render complete!");

    Ok(())
}
