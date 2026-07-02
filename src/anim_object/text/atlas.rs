use std::collections::HashMap;

use cosmic_text::{CacheKey, FontSystem, SwashCache};

use glam::{Vec2, vec2};
#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,

    pub width: f32,
    pub height: f32,

    pub bearing: Vec2,
    pub advance: f32,
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
    pub fn get_glyph_update_data(&mut self) -> Option<GlyphUpdateData> {
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
    pub fn new() -> Self {
        Self {
            glyphs: HashMap::new(),
            dirty: true,

            width: 1024,
            height: 1024,

            pixels: vec![0; 1024 * 1024],

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
            };
        };

        let width = image.placement.width as u32;
        let height = image.placement.height as u32;

        let x = self.cursor_x;
        let y = self.cursor_y;

        for row in 0..height {
            for col in 0..width {
                let src = (row * width + col) as usize;

                let dst = ((y + row) * self.width + x + col) as usize;

                self.pixels[dst] = image.data[src];
            }
        }
        self.cursor_x += width + 1;

        if self.cursor_x + width >= self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }

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
        }
    }
}
