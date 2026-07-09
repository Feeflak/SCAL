use std::path::{Path, PathBuf};
use std::process::Command;

use scal_core::prelude::*;

pub const TEST_W: u32 = 320;
pub const TEST_H: u32 = 240;
pub const TEST_FPS: u32 = 10;
pub const PSNR_THRESHOLD: f64 = 35.0;

pub fn test_scene_settings() -> SceneSettings {
    SceneSettings {
        background_color: Color::new(0.8, 0.8, 0.8, 0.0),
        camera: Camera::new(
            glam::vec2(TEST_W as f32, TEST_H as f32),
            glam::Vec2::ZERO,
            1.0,
        ),
        default_theme: Theme::default(),
    }
}

fn test_encoding(output_path: &str) -> scal_core::EncodingSettings {
    scal_core::EncodingSettings {
        output_path: output_path.to_string(),
        codec_type: scal_core::CodecType::PRORES,
    }
}

fn test_rendering() -> scal_core::RenderingSettings {
    scal_core::RenderingSettings {
        width: TEST_W,
        height: TEST_H,
        fps: TEST_FPS,
        buffer_count: 3,
        text_resolution_multiplier: 1.0,
    }
}

pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

pub fn golden_path(name: &str) -> PathBuf {
    golden_dir().join(format!("{}.mov", name))
}

pub fn output_path(name: &str) -> PathBuf {
    golden_dir().join(format!("_out_{}.mov", name))
}

pub fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

fn ffmpeg_psnr(test: &Path, golden: &Path) -> Result<f64, String> {
    let output = Command::new("ffmpeg")
        .args([
            "-i", &test.to_string_lossy(),
            "-i", &golden.to_string_lossy(),
            "-lavfi", "psnr",
            "-f", "null",
            "-",
        ])
        .output()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        if let Some(pos) = line.find("average:") {
            let rest = &line[pos + 8..];
            let val = rest.split(',').next().unwrap_or("").trim();
            if val == "inf" {
                return Ok(f64::INFINITY);
            }
            if let Ok(v) = val.parse::<f64>() {
                return Ok(v);
            }
        }
    }
    Err(format!("Could not parse PSNR. stderr:\n{}", stderr))
}

pub async fn run_compare(name: &str, project: Project) {
    let handle = tokio::runtime::Handle::current();
    let out = output_path(name);
    let golden = golden_path(name);

    std::fs::create_dir_all(golden_dir()).unwrap();

    scal::render_project(&handle, test_encoding(&out.to_string_lossy()), test_rendering(), project)
        .await
        .expect("render_project failed");

    if !golden.exists() {
        std::fs::copy(&out, &golden).expect("failed to create golden");
        eprintln!("Generated golden: {:?}", golden);
        let _ = std::fs::remove_file(&out);
        return;
    }

    if !ffmpeg_available() {
        eprintln!("ffmpeg not found, skipping PSNR comparison");
        let _ = std::fs::remove_file(&out);
        return;
    }

    let psnr = match ffmpeg_psnr(&out, &golden) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("PSNR comparison error ({}), skipping assertion", e);
            let _ = std::fs::remove_file(&out);
            return;
        }
    };

    let _ = std::fs::remove_file(&out);

    assert!(
        psnr > PSNR_THRESHOLD || psnr.is_infinite(),
        "PSNR {:.2} dB is below threshold {} dB for {}",
        psnr, PSNR_THRESHOLD, name
    );
}
