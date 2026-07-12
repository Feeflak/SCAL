/// Default type for time.
pub type Time = f32;

/// Allows for easy conversions of different number types into the time type(f32).
/// Addition ally implements convenient functions for unit conversions- s ms
pub trait DurationExt {
    /// seconds
    fn s(self) -> Time;
    /// milliseconds
    fn ms(self) -> Time;
}

impl DurationExt for f32 {
    fn s(self) -> Time {
        self
    }
    fn ms(self) -> Time {
        self / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for u32 {
    fn s(self) -> Time {
        self as f32
    }
    fn ms(self) -> Time {
        self as f32 / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for i32 {
    fn s(self) -> Time {
        self as f32
    }
    fn ms(self) -> Time {
        self as f32 / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for u64 {
    fn s(self) -> Time {
        self as f32
    }
    fn ms(self) -> Time {
        self as f32 / 1000.0
    }
}
