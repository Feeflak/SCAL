use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let title = text()
        .spans([
            span("Rich "),
            span("text").color(color(1.0, 0.45, 0.35, 1.0)).bold(),
            span(" with "),
            span("colored").color(color(0.4, 0.85, 0.5, 1.0)),
            span(" and "),
            span("bold").bold(),
            span(" sections"),
        ])
        .font_size(72.0)
        .pos(vec2(960.0, 300.0))
        .build();

    let multiline = text()
        .spans([
            span("Line one is plain\n"),
            span("line two is ").color(color(0.6, 0.7, 1.0, 1.0)),
            span("bold blue").color(color(0.3, 0.5, 1.0, 1.0)).bold(),
        ])
        .font_size(48.0)
        .pos(vec2(960.0, 470.0))
        .build();

    let equation = math(r"\int_0^1 x^2\,dx = \frac{1}{3}")
        .font_size(96.0)
        .pos(vec2(960.0, 690.0))
        .build();

    let euler = math(r"e^{i\pi} + 1 = 0 \qquad \sqrt{a^2 + b^2} = \sum_{k=1}^{n} k")
        .font_size(56.0)
        .color(color(1.0, 0.85, 0.3, 1.0))
        .pos(vec2(960.0, 960.0))
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.03, 0.03, 0.04, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            title.instantiate(),
            multiline.instantiate(),
            equation.instantiate(),
            euler.instantiate(),
            wait(0.5.s()),
            equation
                .color()
                .to(color(0.4, 0.8, 1.0, 1.0))
                .over(1.0.s())
                .ease(Ease::InOutCubic),
            wait(0.5.s()),
        ],
    }
}
