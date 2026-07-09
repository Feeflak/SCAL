use glam::Vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let typing = Sfx {
        path: "./keeb.wav".to_string(),
        volume: 5.,
        pitch: 1.,
        time_offset: 0.,
        duration: 5.,
        pitch_variation: 0.05,
    };

    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    let code = code()
        .source("fn main() {\n    println!(\"Hello, world!\");\n}\n")
        .font_family("SF Pro Display")
        .font_size(20.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .pos(Vec2::new(960., 540.))
        .build();

    let pointer = svg()
        .path("./pointer-tool.svg")
        .size(Vec2::new(40., 40.))
        .color(Color::WHITE)
        .stretch(StretchMode::Fit)
        .pos(Vec2::new(500., 500.))
        .z(1.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 0.),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            code.instantiate(),
            pointer.instantiate(),
            wait(1.s()),
            parallel![
                code.add_lines()
                    .str(
                        r"
fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}
                "
                    )
                    .over(5.s())
                    .style(CodeAnimationStyle::TypeWriterInstantResize),
                AnimOP::PlaySound(typing, 0.),
            ],
            wait(1.s()),
            pointer
                .transform
                .position()
                .to(Vec2::new(0., -3000.))
                .over(0.5.s())
                .ease(Ease::InOutCubic),
            wait(2.s()),
        ],
    }
}

