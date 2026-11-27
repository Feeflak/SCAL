//! Morph one piece of code into another while the font grows, in a single motion.
//!
//! `morph_to` matches the tokens of the old and the new code: the ones in both glide to their
//! new place, the rest fades out or in. `.font_size()` zooms along the way, and the code window
//! resizes with its content.
use glam::{Vec2, vec2};
use scal_core::prelude::*;

const WINDOW: Vec2 = vec2(1920., 1080.);

const BEFORE: &str = "
fn calc_aoa(&self) -> f32 {
    let mut base = self.base().get_transform().clone();
    base.origin = Vector3::ZERO;
    let local_airspeed = base.affine_inverse() * -self.air_speed;
    let aoa = local_airspeed.y.atan2(local_airspeed.z);
    -aoa.to_degrees()
}
";

const AFTER: &str = "
// Remove the component of airflow that runs along the wing's span
let airflow_in_wing_plane = (relative_airflow
    - span_dir * relative_airflow.dot(span_dir))
";

#[scal_ipc::main]
fn main() -> Project {
    let cw = code_window()
        .source(BEFORE)
        .syntax(Syntax::Rust)
        .font_size(32.)
        .title("wing.rs")
        .pos(WINDOW / 2.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.0, 0.0, 0.0, 1.0),
            camera: Camera::new(WINDOW, Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            cw.instantiate(),
            wait(1.0.s()),
            cw.morph_to(AFTER).font_size(40.).over(2.5.s()),
            wait(1.0.s()),
        ],
    }
}
