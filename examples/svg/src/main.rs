use anyhow::Result;
use glam::{Vec2, vec2, vec3};
use log::{LevelFilter, info};
use scal::{
    anim_object::*,
    projection::Camera,
    types::Color,
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Info;
pub const CANVAS_SIZE: Vec2 = vec2(1920., 1080.);

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = colog::default_builder();
    builder.filter_level(LEVEL_FILTER);
    builder.init();
    let handle = Handle::current();

    let encoding_settings = scal::encoder::EncodingSettings {
        output_path: "test.mov".to_string(),
        codec_type: scal::encoder::CodecType::PRORES,
    };
    let rendering_settings = scal::renderer::RenderingSettings {
        camera: Camera::new(CANVAS_SIZE, Vec2::ZERO, 1.),
        background_color: Color::new(0.8, 0.8, 0.8, 0.),
        buffer_count: 3,
        width: 1920,
        height: 1080,
        fps: 60,
        text_resolution_multiplier: 1.0,
    };

    let svg = svg(
        Transform::new(None, vec3(400., 250., 1.), 0., Vec2::ONE),
        "test.svg",
        vec2(800., 500.),
        Color::WHITE,
        Some(Color::GREEN),
        Some(Color::WHITE),
        Some(0.25),
        scal::anim_object::image::StretchMode::Fill,
    );

    scal::run_loop(&handle, encoding_settings, rendering_settings, vec![
        svg.instantiate(),
        wait(1.0),
    ]).await?;
    info!("Hello, world!");
    Ok(())
}
