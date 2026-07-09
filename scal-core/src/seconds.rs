pub type Seconds = f32;

pub trait DurationExt {
    fn s(self) -> Seconds;
    fn ms(self) -> Seconds;
}

impl DurationExt for f32 {
    fn s(self) -> Seconds {
        self
    }
    fn ms(self) -> Seconds {
        self / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for u32 {
    fn s(self) -> Seconds {
        self as f32
    }
    fn ms(self) -> Seconds {
        self as f32 / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for i32 {
    fn s(self) -> Seconds {
        self as f32
    }
    fn ms(self) -> Seconds {
        self as f32 / 1000.0
    }
}

#[allow(clippy::cast_precision_loss)]
impl DurationExt for u64 {
    fn s(self) -> Seconds {
        self as f32
    }
    fn ms(self) -> Seconds {
        self as f32 / 1000.0
    }
}
