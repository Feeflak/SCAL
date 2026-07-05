use std::sync::LazyLock;

use anyhow::Result;

use glam::{Vec2, Vec3, vec2};
use log::{LevelFilter, info};
use scal::{
    anim_object::text::code::{
        Code, CodeAnimationStyle, Syntax,
        theme::{Base16, Theme},
    },
    prelude::*,
    projection::Camera,
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Info;
const THEME: LazyLock<Theme> = LazyLock::new(|| {
    Theme::from_base16(Base16 {
        colors: [
            0x11121d.into(), 0x1A1B2A.into(), 0x212234.into(), 0x282c34.into(),
            0x4a5057.into(), 0xa0a8cd.into(), 0xa0a8cd.into(), 0xa0a8cd.into(),
            0xee6d85.into(), 0xf6955b.into(), 0xd7a65f.into(), 0x95c561.into(),
            0x38a89d.into(), 0x7199ee.into(), 0xa485dd.into(), 0x773440.into(),
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
    let curve = AnimationCurve::Linear;

    // Code object (keep reference for animation)
    let code = Code::new(
        "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
        Syntax::Rust,
        THEME.to_owned(),
        "SF Pro Display Bold".to_string(),
        scal::anim_object::text::Align::Left,
        20.,
        Transform::new(None, Vec3::ZERO, 0., Vec2::ONE),
    );

    let circle_r = 12.0;

    // Traffic lights + title text arranged in a Row inside the title bar
    let c1 = circle(
        Transform::new(None, Vec3::ZERO, 0., Vec2::ONE),
        circle_r,
        Color::new(1.0, 0.373, 0.341, 1.0),
    );
    let c2 = circle(
        Transform::new(None, Vec3::ZERO, 0., Vec2::ONE),
        circle_r,
        Color::new(1.0, 0.741, 0.180, 1.0),
    );
    let c3 = circle(
        Transform::new(None, Vec3::ZERO, 0., Vec2::ONE),
        circle_r,
        Color::new(1.0, 0.373, 0.341, 1.0),
    );
    let title_text = text(
        Transform::new(None, Vec3::ZERO, 0., Vec2::ONE),
        "src/main.rs".to_string(),
        "SF Pro Display Bold".to_string(),
        scal::anim_object::text::Align::Left,
        Color::new(0.812, 0.812, 0.812, 1.0),
        28.,
    );

    // Inner Row layout for title bar contents
    let title_layout = layout(
        Vec3::ZERO,
        PinPoint::C,
        vec![c1.into(), c2.into(), c3.into(), title_text.into()],
        LayoutBackground {
            color: Color::new(0.106, 0.106, 0.106, 1.0),
            corner_radius: 0.5,
        },
        LayoutDir::Row,
        Alignment::Center,
        8.0,
        0.0, 0.0, 28.0, 0.0,
    );

    // Outer Column layout: title_layout background + code
    let result = layout(
        CANVAS_SIZE.extend(0.) / 2.,
        PinPoint::C,
        vec![title_layout.into(), code.clone().into()],
        LayoutBackground {
            color: Color::new(0.176, 0.176, 0.176, 1.0),
            corner_radius: 0.5,
        },
        LayoutDir::Column,
        Alignment::Start,
        0.0,
        0.0, 24.0, 24.0, 24.0,
    );

    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            result.instantiate(),
            wait(1.0),
            code.add_lines(
                "\nfn fib(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fib(n-1) + fib(n-2),\n    }\n}\n".into(),
                4,
                curve.clone(),
                3.0,
                CodeAnimationStyle::TypeWriter,
            ),
            wait(1.0),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
