use glam::Vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let rect = rectangle()
        .size(Vec2::new(600., 400.))
        .corner_radius(40.)
        .color(Color::new(0., 0.2, 0.4, 1.))
        .pos(Vec2::new(400., 540.))
        .build();

    let circle = circle()
        .radius(200.)
        .color(Color::new(0.8, 0.2, 0.2, 1.))
        .pos(Vec2::new(1200., 500.))
        .build();

    let hex = polygon()
        .radius(180.)
        .sides(6)
        .color(Color::new(0.2, 0.7, 0.3, 1.))
        .pos(Vec2::new(800., 300.))
        .build();

    let triangle = polygon()
        .radius(150.)
        .sides(3)
        .color(Color::new(0.9, 0.6, 0.1, 1.))
        .pos(Vec2::new(1600., 700.))
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            rect.instantiate(),
            circle.instantiate(),
            hex.instantiate(),
            triangle.instantiate(),
            wait(1.s()),
            parallel![
                triangle
                    .transform
                    .position()
                    .to(Vec2::new(350., 800.))
                    .over(1.s())
                    .ease(Ease::OutBack),
                rect.transform
                    .position()
                    .to(Vec2::new(960., 540.))
                    .over(1.s())
                    .ease(Ease::InOutBack),
            ],
        ],
    }
}

