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
        let r: u8 = (value & 255) as u8;
        let g: u8 = (value & (255 << 8)) as u8;
        let b: u8 = (value & (255 << 16)) as u8;
        let a: u8 = (value & (255 << 24)) as u8;
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
