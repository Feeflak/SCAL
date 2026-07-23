use glam::Vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let img = image()
        .path("test.png")
        .size(Vec2::new(800., 500.))
        .color(Color::WHITE)
        .stretch(StretchMode::Fill)
        .pos(Vec2::new(400., 250.))
        .z(1.)
        .build();

    let img2 = image()
        .path("test.png")
        .size(Vec2::new(800., 500.))
        .color(Color::WHITE)
        .stretch(StretchMode::Fit)
        .pos(Vec2::new(800., 250.))
        .z(1.)
        .build();

    let img_fit = image()
        .path("test.png")
        .size(Vec2::new(150., 200.))
        .color(color(1., 1., 0.8, 1.))
        .stretch(StretchMode::Fit)
        .pos(Vec2::new(1600., 250.))
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.8, 0.8, 0.8, 0.),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            img.instantiate(),
            img2.instantiate(),
            img_fit.instantiate(),
            wait(1.s()),
        ],
    }
}
