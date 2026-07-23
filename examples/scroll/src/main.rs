use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    // Standalone reference rectangles
    let ref1 = rectangle()
        .size(Vec2::new(300., 100.))
        .corner_radius(10.)
        .color(Color::new(0.2, 0.4, 0.8, 1.0))
        .pos(vec2(300., 200.))
        .build();

    // Single item in scroll
    let item = rectangle()
        .size(Vec2::new(300., 100.))
        .corner_radius(10.)
        .color(Color::new(0.8, 0.2, 0.2, 1.0))
        .build();

    let scroll = scrol_layout()
        .viewport(400., 200.)
        .direction(LayoutDir::Column)
        .gap(0.)
        .padding(0.)
        .background_color(Color::new(0.1, 0.1, 0.1, 1.0))
        .show_scrollbar(false)
        .item(item)
        .pos(vec2(960., 540.))
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            ref1.instantiate(),
            scroll.instantiate(),
            wait(3.s()),
        ],
    }
}
