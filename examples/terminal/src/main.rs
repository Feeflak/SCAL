use glam::{Vec2, vec2};
use scal_core::prelude::*;

const WINDOW: Vec2 = vec2(1920., 1080.);

#[scal_ipc::main]
fn main() -> Project {
    const TERM_WIDTH: f32 = 800.;
    const TERM_HEIGHT: f32 = 500.;

    let term = terminal()
        .shell("fish")
        .prompt("❯ ")
        .font_family("JetBrains Mono")
        .font_size(18.)
        .width(TERM_WIDTH)
        .height(TERM_HEIGHT)
        .background_color(Color::new(0.08, 0.08, 0.08, 1.0))
        .text_color(Color::new(0.8, 0.8, 0.8, 1.0))
        .source_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures"))
        .pos(WINDOW / 2.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            term.instantiate(),
            wait(0.5.s()),
            // Type the first command and show output
            term.input().value("ls -la --color=always").over(0.5.s()),
            wait(0.3.s()),
            term.output().pull_all().over(0.5.s()),
            wait(0.5.s()),
            // Type the second command with visual override
            term.input()
                .value("cargo build")
                .input_view_override("cargo build --release")
                .over(0.8.s()),
            wait(0.3.s()),
            // Show partial output
            term.output().pull(50).over(0.5.s()),
            wait(0.3.s()),
            // Skip some and show the rest
            term.output().skip(10).pull_all().over(0.5.s()),
            wait(0.5.s()),
        ],
    }
}
