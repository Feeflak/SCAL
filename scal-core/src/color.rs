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

/// Convenience function to create a [`Color`] from RGBA float components (0.0–1.0).
/// ```ignore
/// let c = color(1.0, 0.0, 0.0, 1.0);
/// ```
#[must_use]
pub const fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

/// Parse a hex color string (`#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`) into a [`Color`].
/// ```ignore
/// let c = hex("#ff0000");
/// let c = hex("#ff0000ff");
/// ```
#[must_use]
pub fn hex(value: &str) -> Color {
    let bytes = value.as_bytes();
    let (r, g, b, a) = match bytes.len() {
        4 if bytes[0] == b'#' => {
            let r = hex_digit(bytes[1]) * 17;
            let g = hex_digit(bytes[2]) * 17;
            let b = hex_digit(bytes[3]) * 17;
            (r, g, b, 255)
        }
        5 if bytes[0] == b'#' => {
            let r = hex_digit(bytes[1]) * 17;
            let g = hex_digit(bytes[2]) * 17;
            let b = hex_digit(bytes[3]) * 17;
            let a = hex_digit(bytes[4]) * 17;
            (r, g, b, a)
        }
        7 if bytes[0] == b'#' => {
            let r = hex_byte(&value[1..3]);
            let g = hex_byte(&value[3..5]);
            let b = hex_byte(&value[5..7]);
            (r, g, b, 255)
        }
        9 if bytes[0] == b'#' => {
            let r = hex_byte(&value[1..3]);
            let g = hex_byte(&value[3..5]);
            let b = hex_byte(&value[5..7]);
            let a = hex_byte(&value[7..9]);
            (r, g, b, a)
        }
        _ => panic!("invalid hex color: {value}"),
    };
    Color::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    )
}

fn hex_byte(s: &str) -> u8 {
    u8::from_str_radix(s, 16).unwrap_or_else(|_| panic!("invalid hex byte: {s}"))
}

/// Create a [`Color`] from HSV components.
/// - `h`: hue in degrees (0–360)
/// - `s`: saturation (0.0–1.0)
/// - `v`: value/brightness (0.0–1.0)
/// ```ignore
/// let c = hsv(120.0, 1.0, 1.0); // green
/// ```
#[must_use]
pub fn hsv(h: f32, s: f32, v: f32) -> Color {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::new(r + m, g + m, b + m, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_function_creates_color() {
        let c = color(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.r, 0.1);
        assert_eq!(c.g, 0.2);
        assert_eq!(c.b, 0.3);
        assert_eq!(c.a, 0.4);
    }

    #[test]
    fn hex_parses_six_digit() {
        let c = hex("#ff0000");
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g - 0.0).abs() < f32::EPSILON);
        assert!((c.b - 0.0).abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hex_parses_eight_digit() {
        let c = hex("#00ff0080");
        assert!((c.r - 0.0).abs() < f32::EPSILON);
        assert!((c.g - 1.0).abs() < f32::EPSILON);
        assert!((c.b - 0.0).abs() < f32::EPSILON);
        assert!((c.a - 0.50196).abs() < 0.001);
    }

    #[test]
    fn hex_parses_shorthand() {
        let c = hex("#f0f");
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g - 0.0).abs() < f32::EPSILON);
        assert!((c.b - 1.0).abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hsv_red() {
        let c = hsv(0.0, 1.0, 1.0);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 0.0).abs() < 0.001);
        assert!((c.b - 0.0).abs() < 0.001);
        assert!((c.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn hsv_green() {
        let c = hsv(120.0, 1.0, 1.0);
        assert!((c.r - 0.0).abs() < 0.001);
        assert!((c.g - 1.0).abs() < 0.001);
        assert!((c.b - 0.0).abs() < 0.001);
        assert!((c.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn hsv_blue() {
        let c = hsv(240.0, 1.0, 1.0);
        assert!((c.r - 0.0).abs() < 0.001);
        assert!((c.g - 0.0).abs() < 0.001);
        assert!((c.b - 1.0).abs() < 0.001);
        assert!((c.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn hsv_wraps_hue() {
        let c = hsv(360.0, 1.0, 1.0);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 0.0).abs() < 0.001);
        assert!((c.b - 0.0).abs() < 0.001);
    }
}

fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("invalid hex digit: {c}"),
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
