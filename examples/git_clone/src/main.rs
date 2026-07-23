use glam::{Vec2, vec2};
use scal_core::prelude::*;

const WINDOW: Vec2 = vec2(1920., 1080.);

#[scal_ipc::main]
fn main() -> Project {
    const TERM_WIDTH: f32 = 1500.;
    const TERM_HEIGHT: f32 = 800.;

    let term = terminal()
        .shell("fish")
        .prompt("❯ ")
        .font_family("JetBrains Mono Nerd")
        .title_font_size(28.)
        .font_size(22.)
        .width(TERM_WIDTH)
        .height(TERM_HEIGHT)
        .background_color(color(0.08, 0.08, 0.08, 1.0))
        .text_color(color(0.8, 0.8, 0.8, 1.0))
        .source_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures"))
        .pos(WINDOW / 2.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.1, 0.1, 0.1, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            term.instantiate(),
            wait(0.5.s()),
            // Type the first command and show output
            term.input()
                .value("git clone --progress https://github.com/Feeflak/SCAL")
                .over(0.5.s()),
            term.output().pull_all().over(0.3.s()),
            // Type the second command with visual override
            term.input().value("cd ./SCAL").over(0.3.s()),
            // term.input().value("nix develop").over(0.3.s()),
            // term.output().pull_all().over(0.3.s()),
            term.input().value("ping google.com -c 4").over(0.3.s()),
            term.output().pull_line().over(0.2.s()),
            wait(0.5.s()),
            term.output().pull_line().over(0.2.s()),
            wait(0.5.s()),
            term.output().pull_line().over(0.2.s()),
            wait(0.5.s()),
            term.output().pull_line().over(0.2.s()),
            wait(0.5.s()),
            term.output().pull_line().over(0.2.s()),
            wait(0.5.s()),
        ],
    }
}
