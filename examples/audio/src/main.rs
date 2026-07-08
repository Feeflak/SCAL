use std::sync::LazyLock;

use anyhow::Result;
use glam::{Vec2, vec2, vec3};
use log::LevelFilter;
use scal::{
    anim_object::text::code::{
        CodeAnimationStyle, Syntax,
        theme::{Base16, Theme},
    },
    prelude::*,
    projection::Camera,
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Debug;
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
    let typing = Sfx {
        path: "./keeb.wav".to_string(),
        volume: 5., pitch: 1., time_offset: 0., duration: 5., pitch_variation: 0.05,
    };
    let click = Sfx {
        path: "./mouse.mp4".to_string(),
        volume: 3., pitch: 1., time_offset: 0.5, duration: 0.3, pitch_variation: 0.1,
    };

    let mut builder = colog::default_builder();
    builder.filter_level(LEVEL_FILTER);
    builder.init();
    let handle = Handle::current();

    let encoding_settings = scal::encoder::EncodingSettings {
        output_path: "test.mov".to_string(),
        codec_type: scal::encoder::CodecType::H264Nvenc,
    };
    let rendering_settings = scal::renderer::RenderingSettings {
        camera: Camera::new(CANVAS_SIZE, Vec2::ZERO, 1.),
        background_color: Color::new(0.8, 0.8, 0.8, 0.),
        buffer_count: 3,
        width: 3840,
        height: 2160,
        fps: 60,
        text_resolution_multiplier: 2.0,
    };

    let mut cw = code_window(
        CANVAS_SIZE.extend(0.) / 2.,
        "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
        THEME.to_owned(),
        "SF Pro Display Bold".to_string(),
        scal::anim_object::text::Align::Left,
        20.,
        Syntax::Rust,
        "src/main.rs".to_string(),
        1200., 800.,
        28.,
    );
    cw.code.show_line_numbers = true;

    const POINTER_SIZE: f32 = 40.;
    let pointer = svg(
        transform(vec3(500., 500., 1.)),
        "./pointer-tool.svg",
        Vec2::ONE * POINTER_SIZE,
        Color::WHITE,
        None, None, None,
        image::StretchMode::Fit,
    );

    scal::run_loop(&handle, encoding_settings, rendering_settings, vec![
        cw.instantiate(),
        pointer.instantiate(),
        wait(1.0),
        all(vec![
            cw.add_lines(
                r#"
fn fib(n: u32) -> u32 {
    info!("Hello, world!");
    let a = CodeAnimationStyle::TypeWriter;
    let b = vec![1, 2, 3, 4];
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}
                "#, 4,
                AnimationCurve::Linear,
                5.0,
                CodeAnimationStyle::TypeWriterInstantResize,
            ),
            play(typing, 0.),
        ]),
        wait(1.0),
        pointer.transform().position_to_object(
            &cw.close_btn, Vec2::ONE * 15., 1.0, AnimationCurve::EaseOutBack,
        ),
        all(vec![
            sequence(vec![
                cw.close_btn.transform().scale_to(
                    Vec2::ONE * 0.85, 0.2, AnimationCurve::EaseOutCubic,
                ),
                cw.close_btn.transform().scale_to(
                    Vec2::ONE, 0.25, AnimationCurve::EaseOutCubic,
                ),
            ]),
            play(click, 0.),
        ]),
        all(vec![
            cw.position_to_object(&cw.close_btn, Vec2::ZERO, 0.3, AnimationCurve::EaseOutCubic),
            cw.scale_to(Vec2::ZERO, 0.3, AnimationCurve::EaseOutCubic),
        ]),
        pointer.transform().position_to(
            vec2(0., -1.) * 3000., 0.5, AnimationCurve::EaseInOutCubic,
        ),
        wait(2.0),
    ]).await?;
    Ok(())
}
