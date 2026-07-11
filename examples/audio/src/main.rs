use glam::{Vec2, vec2};
use scal_core::prelude::*;

const WINDOW: Vec2 = vec2(1920., 1080.);
#[scal_ipc::main]
fn main() -> Project {
    let typing = sfx()
        .path("./keeb.wav")
        .volume(5.)
        .pitch(1.)
        .skip_time(0.)
        .duration(5.)
        .pitch_variation(0.05);

    let click = sfx()
        .path("./mouse.mp4")
        .volume(5.)
        .pitch(1.)
        .skip_time(0.5)
        .duration(0.5)
        .pitch_variation(0.05);

    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    const CW_WIDTH: f32 = 800.;
    const CW_HEIGHT: f32 = 600.;

    let cw = code_window()
        .source("fn main() {\n    println!(\"Hello, world!\");\n}\n")
        .font_family("SF Pro Display")
        .font_size(20.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .line_numbers(true)
        .title("fib.rs")
        .width(CW_WIDTH)
        .height(CW_HEIGHT)
        .title_font_size(25.)
        .background_color(Color::new(0.15, 0.15, 0.2, 1.))
        .pos(WINDOW / 2.)
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
            cw.instantiate(),
            pointer.instantiate(),
            wait(0.5.s()),
            parallel![
                cw.add_lines()
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
                typing.play(),
            ],
            wait(0.5.s()),
            pointer
                .transform
                .position()
                .object(cw.close_button())
                .to(vec2(15., 15.))
                .over(0.5.s())
                .ease(Ease::InOutCubic),
            parallel![
                sequence![
                    cw.close_button().scale().to(Vec2::ONE * 0.8).over(0.3.s()),
                    cw.close_button().scale().to(Vec2::ONE).over(0.3.s()),
                ],
                click.play(),
            ],
            cw.transform
                .scale()
                .to(Vec2::ZERO)
                .over(0.5)
                .ease(Ease::OutCubic),
            cw.transform
                .position()
                .to((WINDOW - vec2(CW_WIDTH, CW_HEIGHT)) / 2.)
                .over(0.5)
                .ease(Ease::OutCubic),
            wait(0.5.s()),
        ],
    }
}
