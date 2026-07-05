use std::sync::LazyLock;

use anyhow::Result;

use glam::{Vec2, Vec3, vec2, vec3};
use log::{LevelFilter, info};
use scal::{
    anim_object::text::{
        Align,
        code::{Syntax, theme::Theme},
    },
    prelude::*,
    projection::Camera,
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Info;
const THEME: LazyLock<Theme> = LazyLock::new(|| {
    Theme::from_base16(scal::anim_object::text::code::theme::Base16 {
        colors: [
            0x11121d.into(),
            0x1A1B2A.into(),
            0x212234.into(),
            0x282c34.into(),
            0x4a5057.into(),
            0xa0a8cd.into(),
            0xa0a8cd.into(),
            0xa0a8cd.into(),
            0xee6d85.into(),
            0xf6955b.into(),
            0xd7a65f.into(),
            0x95c561.into(),
            0x38a89d.into(),
            0x7199ee.into(),
            0xa485dd.into(),
            0x773440.into(),
        ],
    })
});
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
    };
    let code = code(
        transform(CANVAS_SIZE.extend(0.) / 2.),
        "const t : String = 25;".to_string(),
        THEME.to_owned(),
        "SF Pro Display Bold".to_string(),
        Align::Center,
        55.,
        Syntax::Rust,
        vec![],
    );

    let rect = rectangle(
        transform(vec3(400., 540., 0.)),
        vec2(600., 400.),
        40.,
        Color::new(0., 0.2, 0.4, 1.),
    );

    let circle = circle(
        transform(vec3(1200., 500., 0.)),
        200.,
        Color::new(0.8, 0.2, 0.2, 1.),
    );

    let hex = polygon(
        transform(vec3(800., 300., 0.)),
        180.,
        6,
        Color::new(0.2, 0.7, 0.3, 1.),
    );

    let triangle = polygon(
        transform(vec3(1600., 700., 0.)),
        150.,
        3,
        Color::new(0.9, 0.6, 0.1, 1.),
    );

    let text = text(
        Transform::new(Some(&rect), Vec3::ZERO, 0., Vec2::ONE),
        "const t : String = 25;".to_string(),
        "SF Pro Display Bold".to_string(),
        Align::Center,
        Color::BLACK,
        55.,
    );

    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            code.instantiate(),
            text.instantiate(),
            rect.instantiate(),
            circle.instantiate(),
            hex.instantiate(),
            triangle.instantiate(),
            wait(1.0),
            all(vec![
                triangle
                    .transform()
                    .position_to(vec2(350., 800.), 1., AnimationCurve::EaseOutBack),
                rect.transform()
                    .position_to(vec2(0.5, 0.5), 1., AnimationCurve::EaseOutBack),
            ]),
            (rect
                .transform()
                .position_to(CANVAS_SIZE / 2., 1., AnimationCurve::EaseInOutBack)),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
