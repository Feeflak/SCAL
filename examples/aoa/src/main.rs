//! Rotate a wing around its quarter chord while a label counts the angle of attack.
//!
//! Two pieces make this work for any "rotate something, show the number" animation:
//! - `.pivot(offset)` sets the point an object rotates around (offset from its center).
//! - A `{:.1}` placeholder in `text()`/`math()` plus `.number()` animates the shown number.
//!   Give it the same `.over()` and `.ease()` as the rotation and the two stay in sync.
use glam::vec2;
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let centre = vec2(960.0, 540.0);
    let wing_size = vec2(800.0, 160.0);
    let sky = color(0.30, 0.67, 0.97, 1.0);
    let marker = color(1.0, 0.90, 0.25, 1.0);

    // The airfoil's leading edge is on the left, so the quarter chord is a quarter of the way
    // in: (0.25 - 0.5) * width left of the center.
    let wing = svg()
        .path(concat!(env!("CARGO_MANIFEST_DIR"), "/airfoil.svg"))
        .size(wing_size)
        .pos(centre)
        .pivot(vec2(-0.25 * wing_size.x, 0.0))
        .z(1.0)
        .build();

    // Free stream: a thin line through the pivot that the wing's chord is measured against.
    let pivot_world = centre + vec2(-0.25 * wing_size.x, 0.0);
    let airflow = rectangle()
        .size(vec2(1100.0, 3.0))
        .color(sky)
        .pos(pivot_world)
        .build();

    // The same label as LaTeX and as plain text. `\hphantom` keeps the width constant so the
    // label does not shift while the digits change.
    let label = math(r"\alpha = \textcolor{a}{{:.1}}^\circ\hphantom{0}")
        .var_color("a", marker)
        .font_size(96.0)
        .pos(vec2(960.0, 880.0))
        .build();
    let plain = text()
        .value("AoA {:+.1}°")
        .align(Alignment::Start)
        .font_size(48.0)
        .pos(vec2(120.0, 100.0))
        .build();

    // Positive rotation is clockwise on screen (y points down). The leading edge is on the
    // left, so a clockwise turn lifts the nose: a positive angle of attack is a positive
    // rotation. Facing the other way, you would use `-aoa`.
    let aoa = 15.0;
    let over = 3.0.s();
    let ease = Ease::InOutCubic;

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.05, 0.06, 0.08, 1.0),
            camera: Camera::new(vec2(1920.0, 1080.0), glam::Vec2::ZERO, 1.0),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            airflow.instantiate(),
            wing.instantiate(),
            label.instantiate(),
            plain.instantiate(),
            wait(0.5.s()),
            parallel![
                wing.transform.rotation().to(aoa).over(over).ease(ease),
                label.number().from(0.0).to(aoa).over(over).ease(ease),
                plain.number().from(0.0).to(aoa).over(over).ease(ease),
            ],
            wait(0.5.s()),
            // `from` is optional: without it the count starts from the number being shown.
            parallel![
                wing.transform.rotation().to(0.0).over(over).ease(ease),
                label.number().to(0.0).over(over).ease(ease),
                plain.number().to(0.0).over(over).ease(ease),
            ],
            wait(0.5.s()),
        ],
    }
}
