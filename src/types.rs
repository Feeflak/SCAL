#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
}
impl From<Color> for cosmic_text::Color {
    fn from(color: Color) -> Self {
        let r = (color.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (color.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (color.b.clamp(0.0, 1.0) * 255.0) as u8;
        let a = (color.a.clamp(0.0, 1.0) * 255.0) as u8;

        cosmic_text::Color::rgba(r, g, b, a)
    }
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

pub type Seconds = f32;

#[derive(Clone, Debug)]
pub struct Sfx {
    pub path: String,
    pub volume: f32,
    pub pitch: f32,
    pub time_offset: Seconds,
    pub duration: Seconds,
    pub pitch_variation: f32,
}

impl Sfx {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            volume: 1.0,
            pitch: 1.0,
            time_offset: 0.0,
            duration: 0.0,
            pitch_variation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_u32_no_alpha() {
        let c: Color = 0xFF0000.into();
        assert!((c.r - 1.0).abs() < f32::EPSILON, "r={}", c.r);
        assert!((c.g).abs() < f32::EPSILON, "g={}", c.g);
        assert!((c.b).abs() < f32::EPSILON, "b={}", c.b);
        assert!((c.a - 1.0).abs() < f32::EPSILON, "a={}", c.a);
    }

    #[test]
    fn color_from_u32_with_alpha() {
        let c: Color = 0x80FF0000.into();
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g).abs() < f32::EPSILON);
        assert!((c.b).abs() < f32::EPSILON);
        assert!((c.a - 0.5019608).abs() < 0.001, "a={}", c.a);
    }

    #[test]
    fn color_from_u32_zero() {
        let c: Color = 0x000000.into();
        assert!((c.r).abs() < f32::EPSILON);
        assert!((c.g).abs() < f32::EPSILON);
        assert!((c.b).abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_constants() {
        assert!((Color::TRANSPARENT.a).abs() < f32::EPSILON);
        assert!((Color::BLACK.r).abs() < f32::EPSILON);
        assert!((Color::BLACK.a - 1.0).abs() < f32::EPSILON);
        assert!((Color::WHITE.r - 1.0).abs() < f32::EPSILON);
        assert!((Color::WHITE.a - 1.0).abs() < f32::EPSILON);
        assert!((Color::RED.r - 1.0).abs() < f32::EPSILON);
        assert!((Color::RED.g).abs() < f32::EPSILON);
        assert!((Color::RED.b).abs() < f32::EPSILON);
        assert!((Color::GREEN.g - 1.0).abs() < f32::EPSILON);
        assert!((Color::BLUE.b - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_into_cosmic_text() {
        let c = Color::new(1.0, 0.5, 0.0, 0.25);
        let ct: cosmic_text::Color = c.into();
        assert_eq!(ct.r(), 255);
        assert_eq!(ct.g(), 127);
        assert_eq!(ct.b(), 0);
        assert_eq!(ct.a(), 63);
    }

    #[test]
    fn color_into_cosmic_text_clamp_negative() {
        let c = Color::new(-0.5, 0.0, 0.0, 1.0);
        let ct: cosmic_text::Color = c.into();
        assert_eq!(ct.r(), 0);
    }

    #[test]
    fn color_into_cosmic_text_clamp_overflow() {
        let c = Color::new(1.5, 0.0, 0.0, 1.0);
        let ct: cosmic_text::Color = c.into();
        assert_eq!(ct.r(), 255);
    }

    #[test]
    fn sfx_new_defaults() {
        let sfx = Sfx::new("path/to/sound.wav");
        assert_eq!(sfx.path, "path/to/sound.wav");
        assert!((sfx.volume - 1.0).abs() < f32::EPSILON);
        assert!((sfx.pitch - 1.0).abs() < f32::EPSILON);
        assert!((sfx.time_offset).abs() < f32::EPSILON);
        assert!((sfx.duration).abs() < f32::EPSILON);
        assert!((sfx.pitch_variation).abs() < f32::EPSILON);
    }
}
