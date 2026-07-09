mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_svg() {
    let svg_path = project_root().join("examples").join("svg").join("test.svg");

    let s = svg()
        .path(svg_path.to_string_lossy().to_string())
        .size(glam::vec2(200., 150.))
        .color(Color::WHITE)
        .fill(Color::GREEN)
        .stroke(Color::WHITE)
        .stroke_width(0.25)
        .stretch(StretchMode::Fill)
        .pos(glam::vec2(160., 120.))
        .z(1.)
        .build();

    run_compare("svg", Project {
        scene_settings: test_scene_settings(),
        timeline: timeline![
            s.instantiate(),
            wait(0.5),
        ],
    }).await;
}
