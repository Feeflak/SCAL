mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_code_window() {
    #[allow(clippy::large_digit_groups)]
    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    let cw = code_window()
        .source("fn main() {\n    println!(\"hi\");\n}\n")
        .font_family("monospace")
        .font_size(12.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .line_numbers(true)
        .title("src/main.rs")
        .width(260.)
        .height(180.)
        .title_font_size(16.)
        .pos(glam::vec2(200., 120.))
        .build();

    let svg_path = project_root()
        .join("examples")
        .join("code_window")
        .join("pointer-tool.svg");
    let pointer = svg()
        .path(svg_path.to_string_lossy().to_string())
        .size(glam::Vec2::ONE * 24.)
        .color(Color::WHITE)
        .stretch(StretchMode::Fit)
        .pos(glam::vec2(50., 50.))
        .z(1.)
        .build();

    run_compare("code_window", Project {
        scene_settings: test_scene_settings(),
        timeline: timeline![
            cw.instantiate(),
            pointer.instantiate(),
            wait(0.3),
            cw.add_lines()
                .str("fn fib(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fib(n-1) + fib(n-2),\n    }\n}\n")
                .from_line(4)
                .over(1.0)
                .style(CodeAnimationStyle::TypeWriter),
            wait(0.3),
            pointer.transform.position()
                .object(cw.close_button())
                .to(glam::Vec2::ONE * 10.)
                .over(0.8)
                .ease(Ease::OutBack),
            wait(0.3),
        ],
    }).await;
}
