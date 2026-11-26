use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    // Dark rounded background panel.
    let panel = rectangle()
        .size(vec2(1400.0, 720.0))
        .corner_radius(48.0)
        .color(color(0.08, 0.10, 0.14, 1.0))
        .pos(vec2(960.0, 540.0))
        .hidden()
        .build();

    // Reusable template for small labels.
    let label_tmpl = text_template()
        .font_family("sans-serif")
        .font_size(52.0)
        .color(color(0.72, 0.76, 0.84, 1.0))
        .align(Alignment::Center)
        .modifier(
            text_modifier()
                .thickness(1.0)
                .softness(8.0)
                .color(color(0.0, 0.0, 0.0, 0.35))
                .pos(vec2(4.0, 4.0))
                .build(),
        )
        .build();

    // Big title with glow + outline + drop shadow.
    let title = text()
        .value("SCAL")
        .pos(vec2(960.0, 600.0))
        .hidden()
        .font_size(260.0)
        .font_family("sans-serif")
        .color(color(0.95, 0.97, 1.0, 1.0))
        .align(Alignment::Center)
        // Soft blue glow.
        .modifier(
            text_modifier()
                .thickness(0.0)
                .softness(40.0)
                .color(color(0.25, 0.55, 0.95, 0.4))
                .build(),
        )
        // Hard blue outline.
        .modifier(
            text_modifier()
                .thickness(7.0)
                .color(color(0.2, 0.5, 0.9, 1.0))
                .build(),
        )
        // Drop shadow.
        .modifier(
            text_modifier()
                .thickness(2.0)
                .softness(14.0)
                .color(color(0.0, 0.0, 0.0, 0.55))
                .pos(vec2(14.0, 14.0))
                .build(),
        )
        .build();
    let title_handle = TextHandle(title.id);

    // Subtitle created from the template, starts transparent and lower.
    let subtitle = label_tmpl
        .text()
        .value("Smooth Code Animation Library")
        .pos(vec2(960.0, 740.0))
        .color(color(0.72, 0.76, 0.84, 0.0))
        .build();
    let subtitle_handle = TextHandle(subtitle.id);

    // Feature label from template with selective overrides.
    let feature = label_tmpl
        .text()
        .value("Built with Rust + WGPU")
        .pos(vec2(960.0, 840.0))
        .scale(0.5)
        .rot(-10.0)
        .color(color(0.42, 0.82, 0.62, 1.0))
        .build();
    let feature_handle = TextHandle(feature.id);

    // A tiny decorative badge.
    let badge = rectangle()
        .size(vec2(220.0, 52.0))
        .corner_radius(26.0)
        .color(color(0.2, 0.5, 0.9, 1.0))
        .pos(vec2(960.0, 305.0))
        .hidden()
        .build();

    let pop = Motion::new(0.45, Ease::OutBack);
    let pulse = Motion::new(0.35, Ease::InOutCubic);
    let slide = Motion::new(0.6, Ease::OutCubic);

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.03, 0.03, 0.04, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            // Instantiate everything first.
            spawn![panel, badge, title, subtitle, feature],
            // Panel and title scale in together.
            parallel![
                panel.pop_in().over(0.8),
                title_handle.scale_to(1.0).with(pop).over(0.9),
                title_handle.move_to(vec2(960.0, 420.0)).with(pop).over(0.9),
            ],
            wait(0.15.s()),
            // Badge pops in under the title.
            badge.pop_in().over(0.45),
            wait(0.15.s()),
            // Subtitle slides up and fades in.
            parallel![
                subtitle_handle.move_to(vec2(960.0, 620.0)).with(slide),
                subtitle_handle
                    .color()
                    .to(color(0.72, 0.76, 0.84, 1.0))
                    .with(slide),
            ],
            wait(0.25.s()),
            // Feature label pops in with a little rotation.
            parallel![
                feature_handle.scale_to(1.0).with(pop),
                feature_handle.rotate_to(0.0).with(pop),
            ],
            wait(0.6.s()),
            // Title color pulse + subtle scale bounce.
            parallel![
                title_handle
                    .color()
                    .to(color(0.55, 0.8, 1.0, 1.0))
                    .with(pulse),
                title_handle.scale_to(1.06).with(pulse),
            ],
            parallel![
                title_handle
                    .color()
                    .to(color(0.95, 0.97, 1.0, 1.0))
                    .with(pulse),
                title_handle.scale_to(1.0).with(pulse),
            ],
            wait(1.0.s()),
        ],
    }
}
