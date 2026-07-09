mod common;
use common::*;
use scal_core::prelude::*;

#[tokio::test]
async fn golden_simple() {
    let rect = rectangle()
        .size(glam::vec2(80., 60.))
        .corner_radius(4.)
        .color(Color::new(0., 0.2, 0.4, 1.))
        .pos(glam::vec2(100., 120.))
        .build();

    let circ = circle()
        .radius(40.)
        .color(Color::new(0.8, 0.2, 0.2, 1.))
        .pos(glam::vec2(240., 160.))
        .build();

    let poly = polygon()
        .radius(40.)
        .sides(6)
        .color(Color::new(0.2, 0.7, 0.3, 1.))
        .pos(glam::vec2(160., 80.))
        .build();

    run_compare(
        "simple",
        Project {
            scene_settings: test_scene_settings(),
            timeline: timeline![
                rect.instantiate(),
                circ.instantiate(),
                poly.instantiate(),
                wait(0.3),
                rect.transform
                    .position()
                    .to(glam::vec2(200., 120.))
                    .over(0.8)
                    .ease(Ease::OutBack),
                wait(0.3),
            ],
        },
    )
    .await;
}
