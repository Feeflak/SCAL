use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let text = text()
        .value("TEXT 1")
        .pos(vec2(300.0, 100.0))
        .font_size(160.)
        .color(Color::BLACK)
        // Shadow
        .modifier(
            text_modifier()
                .thickness(1.5)
                .softness(10.0)
                .color(Color::new(0.0, 0.0, 0.0, 0.4))
                .pos(vec2(15.0, 15.0))
                .build(),
        )
        // Red Border
        .modifier(text_modifier().thickness(5.0).color(Color::RED).build())
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![text.instantiate(), wait(1.s()),],
    }
}
