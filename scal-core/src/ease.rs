use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
/// Simple way to make your animations smooth. <https://easings.net/>
#[allow(missing_docs)]
pub enum Ease {
    Linear,
    OutCubic,
    InOutCubic,
    InOutBack,
    OutBack,
    InBack,
}

impl Ease {
    #[must_use]
    /// Apply easing on the t(it should be in a 0..1 range)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::OutCubic => 1.0 - (1.0 - t).powi(3),
            Self::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0f32).mul_add(t, 2.0).powi(3) / 2.0
                }
            }
            Self::InOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    let x = 2.0 * t;
                    (x * x * (C2 + 1.0).mul_add(x, -C2)) / 2.0
                } else {
                    let x = 2.0f32.mul_add(t, -2.0);
                    f32::midpoint(x * x * (C2 + 1.0).mul_add(x, C2), 2.0)
                }
            }
            Self::OutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                let x = t - 1.0;
                (C1 * x).mul_add(x, (C3 * x * x).mul_add(x, 1.0))
            }
            Self::InBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                (C3 * t * t).mul_add(t, -(C1 * t * t))
            }
        }
    }
}
