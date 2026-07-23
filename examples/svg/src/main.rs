use glam::Vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let svg = svg()
        .path("test.svg")
        .size(Vec2::new(800., 500.))
        .color(Color::WHITE)
        .fill(Color::GREEN)
        .stroke(Color::WHITE)
        .stroke_width(0.25)
        .stretch(StretchMode::Fill)
        .pos(Vec2::new(400., 250.))
        .z(1.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.8, 0.8, 0.8, 0.),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![svg.instantiate(), wait(1.s()),],
    }
}
