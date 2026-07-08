use glam::vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let rect = rectangle()
        .size(vec2(600., 400.))
        .corner_radius(40.)
        .color(Color::new(0., 0.2, 0.4, 1.))
        .create(vec2(400., 540.));

    let circle = circle()
        .radius(200.)
        .color(Color::new(0.8, 0.2, 0.2, 1.))
        .create(vec2(1200., 500.));

    let hex = polygon()
        .radius(180.)
        .sides(6)
        .color(Color::new(0.2, 0.7, 0.3, 1.))
        .create(vec2(800., 300.));

    let triangle = polygon()
        .radius(150.)
        .sides(3)
        .color(Color::new(0.9, 0.6, 0.1, 1.))
        .create(vec2(1600., 700.));

    let text = text()
        .value("Hello, SCAL!")
        .font_family("SF Pro Display Bold")
        .font_size(55.)
        .color(Color::BLACK)
        .create(vec2(960., 540.));

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(vec2(1920., 1080.), glam::Vec2::ZERO, 1.),
        },
        timeline: vec![
            rect.instantiate(),
            circle.instantiate(),
            hex.instantiate(),
            triangle.instantiate(),
            text.instantiate(),
            wait(1.s()),
            parallel![
                triangle
                    .transform
                    .position(vec2(350., 800.))
                    .over(1.s())
                    .ease(Ease::OutBack),
                rect.transform
                    .position(vec2(0.5, 0.5))
                    .over(1.s())
                    .ease(Ease::OutBack),
            ],
            rect.transform
                .position(vec2(960., 540.))
                .over(1.s())
                .ease(Ease::InOutBack),
        ],
    }
}
