mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_code() {
    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    let c = code()
        .source("fn main() {\n    println!(\"hi\");\n}\n")
        .font_family("monospace")
        .font_size(14.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .padding(8.)
        .pos(glam::vec2(40., 200.))
        .build();

    run_compare(
        "code",
        Project {
            scene_settings: test_scene_settings(),
            timeline: timeline![
                c.instantiate(),
                wait(0.3),
                c.add_lines()
                    .str("let x = 1;\nlet y = 2;\n")
                    .from_line(3)
                    .over(0.8)
                    .style(CodeAnimationStyle::TypeWriter),
                wait(0.3),
            ],
        },
    )
    .await;
}
