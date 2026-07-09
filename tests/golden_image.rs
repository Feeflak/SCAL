mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_image() {
    let img_path = project_root().join("examples").join("image").join("test.png");

    let img1 = image()
        .path(img_path.to_string_lossy().to_string())
        .size(glam::vec2(100., 80.))
        .color(Color::new(1., 1., 1., 1.))
        .stretch(StretchMode::Fill)
        .pos(glam::vec2(80., 120.))
        .z(1.)
        .build();

    let img2 = image()
        .path(img_path.to_string_lossy().to_string())
        .size(glam::vec2(60., 80.))
        .color(Color::new(1., 1., 0.8, 1.))
        .stretch(StretchMode::Fit)
        .pos(glam::vec2(240., 120.))
        .build();

    run_compare("image", Project {
        scene_settings: test_scene_settings(),
        timeline: timeline![
            img1.instantiate(),
            img2.instantiate(),
            wait(0.5),
        ],
    }).await;
}
