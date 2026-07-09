use glam::{Vec2, vec2};
use scal_core::prelude::*;

const WINDOW: Vec2 = vec2(1920., 1080.);
const SOURCE: &str = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
const NEW_LINES: &str = r"
fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}
";

#[scal_ipc::main]
fn main() -> Project {
    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    let cw = code_window()
        .source(SOURCE)
        .font_family("SF Pro Display")
        .font_size(20.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .line_numbers(true)
        .title("fib.rs")
        .width(800.)
        .height(600.)
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
            camera: Camera::new(WINDOW, Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            cw.instantiate(),
            pointer.instantiate(),
            wait(1.s()),
            cw.add_lines().str(NEW_LINES).from_line(4).over(5.s()),
            wait(1.s()),
            pointer
                .transform
                .position()
                .to(Vec2::new(0., -3000.))
                .over(0.5.s())
                .ease(Ease::InOutCubic),
        ],
    }
}

