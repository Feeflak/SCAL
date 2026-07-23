use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    // Standalone reference rectangles
    // let ref1 = rectangle()
    //     .size(Vec2::new(300., 100.))
    //     .corner_radius(10.)
    //     .color(color(0.2, 0.4, 0.8, 1.0))
    //     .pos(vec2(300., 200.))
    //     .build();

    let item = rectangle()
        .size(Vec2::new(300., 500.))
        .corner_radius(10.)
        .color(color(0.8, 0.2, 0.2, 1.0))
        .build();
    let item2 = rectangle()
        .size(Vec2::new(300., 500.))
        .corner_radius(10.)
        .color(color(0.2, 0.8, 0.2, 1.0))
        .build();

    let item3 = rectangle()
        .size(Vec2::new(300., 500.))
        .corner_radius(10.)
        .color(color(0.2, 0.2, 0.8, 1.0))
        .build();

    let scroll = scrol_layout()
        .viewport(800., 500.)
        .justify(Alignment::Start)
        .direction(LayoutDir::Column)
        .gap(20.)
        .padding(20.)
        .background_color(color(0.1, 0.1, 0.1, 1.0))
        .show_scrollbar(true)
        .corner_radius(20.)
        .item(item)
        .item(item2)
        .item(
            layout()
                .background_color(color(1., 1., 1., 0.1))
                .item(item3)
                .item(text().value("Some Layout Test O:").font_size(50.).build())
                .build(),
        )
        .item(text().value("Some\n Text Test :D").font_size(50.).build())
        .item(
            text()
                .value("Some Text Test \n\n\n2 :|")
                .font_size(50.)
                .build(),
        )
        .pos(vec2(960., 540.))
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.8, 0.8, 0.8, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },

        timeline: timeline![
            scroll.instantiate(),
            wait(0.3.s()),
            scroll.scroll().percent(0.5).over(1.s()).ease(Ease::OutBack),
            wait(0.3.s()),
            scroll
                .scroll()
                .px(100.)
                .over(0.5.s())
                .ease(Ease::InOutCubic),
            wait(0.3.s()),
            scroll.scroll().px(0.).over(0.5.s()).ease(Ease::InOutCubic),
        ],
    }
}
