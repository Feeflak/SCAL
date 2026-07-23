use std::collections::HashMap;

use cosmic_text::{CacheKey, FontSystem, SwashCache};
use swash::scale::image::Content;

use glam::{Vec2, vec2};
#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,

    pub width: f32,
    pub height: f32,

    pub bearing: Vec2,
    pub advance: f32,

    pub is_color: bool,
}

pub struct GlyphAtlas {
    pub glyphs: HashMap<CacheKey, GlyphInfo>,
    pub dirty: bool,

    pub width: u32,
    pub height: u32,

    pub pixels: Vec<u8>,

    pub cursor_x: u32,
    pub cursor_y: u32,
    pub row_height: u32,

    pub cache: SwashCache,
}

pub struct GlyphUpdateData<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}
impl GlyphAtlas {
    pub fn get_glyph_update_data(&mut self) -> Option<GlyphUpdateData<'_>> {
        if self.dirty {
            self.dirty = false;
            Some(GlyphUpdateData {
                width: self.width,
                height: self.height,
                pixels: &self.pixels,
            })
        } else {
            None
        }
    }
    pub fn new(scale: f32) -> Self {
        let size = (1024.0 * scale).ceil() as usize;
        Self {
            glyphs: HashMap::new(),
            dirty: true,

            width: size as u32,
            height: size as u32,

            pixels: vec![0; size * size * 4],

            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,

            cache: SwashCache::new(),
        }
    }

    pub fn get_or_insert(&mut self, font_system: &mut FontSystem, key: CacheKey) -> GlyphInfo {
        if let Some(glyph) = self.glyphs.get(&key) {
            return *glyph;
        }

        let glyph = self.rasterize_glyph(font_system, key);

        self.glyphs.insert(key, glyph);
        self.dirty = true;

        glyph
    }

    fn rasterize_glyph(&mut self, font_system: &mut FontSystem, cache_key: CacheKey) -> GlyphInfo {
        let image = self.cache.get_image(font_system, cache_key);

        let Some(image) = image else {
            return GlyphInfo {
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ZERO,
                width: 0.0,
                height: 0.0,
                bearing: Vec2::ZERO,
                advance: 0.0,
                is_color: false,
            };
        };

        let width = image.placement.width as u32;
        let height = image.placement.height as u32;
        let is_color = image.content == Content::Color || image.content == Content::SubpixelMask;

        if self.cursor_x + width >= self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }

        let x = self.cursor_x;
        let y = self.cursor_y;

        for row in 0..height {
            for col in 0..width {
                let dst = ((y + row) * self.width + x + col) as usize * 4;
                if is_color {
                    // Color emoji: data is RGBA (4 bytes/pixel), store directly
                    let src = ((row * width + col) * 4) as usize;
                    self.pixels[dst] = image.data[src];
                    self.pixels[dst + 1] = image.data[src + 1];
                    self.pixels[dst + 2] = image.data[src + 2];
                    self.pixels[dst + 3] = image.data[src + 3];
                } else {
                    // Alpha mask: data is single channel (1 byte/pixel), set RGB to white
                    let src = (row * width + col) as usize;
                    self.pixels[dst] = 255;
                    self.pixels[dst + 1] = 255;
                    self.pixels[dst + 2] = 255;
                    self.pixels[dst + 3] = image.data[src];
                }
            }
        }
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);

        GlyphInfo {
            uv_min: vec2(x as f32 / self.width as f32, y as f32 / self.height as f32),
            uv_max: vec2(
                (x + width) as f32 / self.width as f32,
                (y + height) as f32 / self.height as f32,
            ),
            width: width as f32,
            height: height as f32,
            bearing: vec2(image.placement.left as f32, image.placement.top as f32),
            advance: image.placement.width as f32,
            is_color,
        }
    }
}
