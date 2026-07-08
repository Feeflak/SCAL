use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Ease {
    Linear,
    OutCubic,
    InOutCubic,
    InOutBack,
    OutBack,
    InBack,
}

impl Ease {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,
            Ease::OutCubic => 1.0 - (1.0 - t).powi(3),
            Ease::InOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
            Ease::InOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    let x = 2.0 * t;
                    (x * x * ((C2 + 1.0) * x - C2)) / 2.0
                } else {
                    let x = 2.0 * t - 2.0;
                    (x * x * ((C2 + 1.0) * x + C2) + 2.0) / 2.0
                }
            }
            Ease::OutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                let x = t - 1.0;
                1.0 + C3 * x * x * x + C1 * x * x
            }
            Ease::InBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
        }
    }
}
