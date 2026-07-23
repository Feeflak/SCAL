mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_code() {
    #[allow(clippy::large_digit_groups)]
    let theme = Theme::from_base16(Base16::from_hex(
        "#11121d #1A1B2A #212234 #282c34 #4a5057 #a0a8cd #a0a8cd #a0a8cd \
         #ee6d85 #f6955b #d7a65f #95c561 #38a89d #7199ee #a485dd #773440",
    ));

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
