use serde::{Deserialize, Serialize};

use crate::ease::Ease;

/// How the head of a vector looks.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum HeadStyle {
    /// A solid triangle.
    #[default]
    Filled,
    /// An open `>` made of two strokes as thick as the line, with rounded corners and ends.
    Rounded,
}

/// A looping animation of the vector head sliding back along the line and returning, to hint at
/// the direction the vector moves in. The line follows the head. Needs some room on the last
/// segment, the head never slides past the point before it.
///
/// One cycle is: slide back and forth over `period` seconds, then rest for `pause` seconds.
/// ```ignore
/// vector()
///     .points([vec2(400., 800.), vec2(900., 500.)])
///     .head_animation(
///         HeadAnimation::new()
///             .distance(30.0)
///             .period(0.5)
///             .pause(0.4)
///             .ease(Ease::OutCubic),
///     )
///     .build()
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeadAnimation {
    /// How far the head slides back in pixels, `None` for 40% of the head length.
    pub distance: Option<f32>,
    /// Seconds one back and forth takes.
    pub period: f32,
    /// Seconds of rest between two back and forths.
    pub pause: f32,
    /// Seconds before the first back and forth starts, from the start of the animation.
    pub delay: f32,
    /// How many back and forths to do, `None` to go on forever.
    pub repeat: Option<u32>,
    /// Easing of the way back and of the way forth, each over half of the `period`.
    pub ease: Ease,
}

impl Default for HeadAnimation {
    fn default() -> Self {
        Self {
            distance: None,
            period: 0.6,
            pause: 0.0,
            delay: 0.0,
            repeat: None,
            ease: Ease::InOutCubic,
        }
    }
}

impl HeadAnimation {
    /// A back and forth every 0.6 seconds, 40% of the head length far, going on forever.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// How far the head slides back in pixels (default 40% of the head length)
    #[must_use]
    pub const fn distance(mut self, distance: f32) -> Self {
        self.distance = Some(distance);
        self
    }
    /// Seconds one back and forth takes (default 0.6)
    #[must_use]
    pub const fn period(mut self, seconds: f32) -> Self {
        self.period = seconds;
        self
    }
    /// Seconds of rest between two back and forths (default 0)
    #[must_use]
    pub const fn pause(mut self, seconds: f32) -> Self {
        self.pause = seconds;
        self
    }
    /// Seconds before the first back and forth (default 0)
    #[must_use]
    pub const fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }
    /// Stop after this many back and forths (default: never stop)
    #[must_use]
    pub const fn repeat(mut self, times: u32) -> Self {
        self.repeat = Some(times);
        self
    }
    /// Easing of the way back and of the way forth (default [`Ease::InOutCubic`])
    #[must_use]
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    /// How far back the head is `time` seconds into the animation, as a fraction of the
    /// distance: 0 is the head at its place, 1 all the way back.
    #[must_use]
    pub fn progress(&self, time: f32) -> f32 {
        let t = time - self.delay;
        if t < 0.0 || self.period <= 0.0 {
            return 0.0;
        }
        let cycle = self.period + self.pause.max(0.0);
        if self.repeat.is_some_and(|n| t >= n as f32 * cycle) {
            return 0.0;
        }
        let in_cycle = t % cycle;
        if in_cycle >= self.period {
            return 0.0;
        }
        let half = in_cycle / self.period;
        // 0 -> 1 -> 0 over one period
        let triangle = 1.0 - (2.0 * half - 1.0).abs();
        self.ease.apply(triangle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear() -> HeadAnimation {
        HeadAnimation::new().period(1.0).ease(Ease::Linear)
    }

    #[test]
    fn goes_back_and_forth_in_one_period() {
        let a = linear();
        assert_eq!(a.progress(0.0), 0.0);
        assert!((a.progress(0.25) - 0.5).abs() < 1e-6);
        assert!((a.progress(0.5) - 1.0).abs() < 1e-6);
        assert!((a.progress(0.75) - 0.5).abs() < 1e-6);
        assert!(a.progress(1.0).abs() < 1e-6);
        // and loops
        assert!((a.progress(3.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rests_during_the_pause_and_before_the_delay() {
        let a = linear().pause(1.0).delay(2.0);
        assert_eq!(a.progress(1.0), 0.0);
        assert!((a.progress(2.5) - 1.0).abs() < 1e-6);
        assert_eq!(a.progress(3.5), 0.0);
        assert!((a.progress(4.5) - 1.0).abs() < 1e-6, "second cycle");
    }

    #[test]
    fn stops_after_repeat() {
        let a = linear().repeat(2);
        assert!((a.progress(1.5) - 1.0).abs() < 1e-6);
        assert_eq!(a.progress(2.5), 0.0);
    }

    #[test]
    fn zero_period_does_not_move() {
        assert_eq!(linear().period(0.0).progress(1.0), 0.0);
    }
}
