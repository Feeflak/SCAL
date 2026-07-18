use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, PartialEq, bytemuck::Pod, bytemuck::Zeroable,
)]

/// Simple RGBA color struct with some helper functions. each field should be a 0..255 float.
#[allow(missing_docs)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// ```ignore
    /// Color {
    ///     r: 0.0,
    ///     g: 0.0,
    ///     b: 0.0,
    ///     a: 0.0,
    /// }
    /// ```
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    /// ```ignore
    /// Color {
    ///     r: 0.0,
    ///     g: 0.0,
    ///     b: 0.0,
    ///     a: 1.0,
    /// }
    /// ```
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// ```ignore
    /// Color {
    ///     r: 1.0,
    ///     g: 1.0,
    ///     b: 1.0,
    ///     a: 1.0,
    /// }
    /// ```
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// ```ignore
    /// Color {
    ///     r: 1.0,
    ///     g: 0.0,
    ///     b: 0.0,
    ///     a: 1.0,
    /// }
    /// ```
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// ```ignore
    /// Color {
    ///     r: 0.0,
    ///     g: 1.0,
    ///     b: 0.0,
    ///     a: 1.0,
    /// }
    /// ```
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    /// ```ignore
    /// Color {
    ///     r: 0.0,
    ///     g: 0.0,
    ///     b: 1.0,
    ///     a: 1.0,
    /// }
    /// ```
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        let r = ((value >> 16) & 0xFF) as u8;
        let g = ((value >> 8) & 0xFF) as u8;
        let b = (value & 0xFF) as u8;
        let a = if (value >> 24) == 0 {
            255
        } else {
            ((value >> 24) & 0xFF) as u8
        };
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }
}
