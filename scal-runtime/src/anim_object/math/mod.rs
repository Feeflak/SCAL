use anyhow::{Context, Result, anyhow};
use glam::Vec2;
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_svg::{SvgColorSyntax, SvgOptions, render_to_svg_with_color_syntax};
use ratex_types::color::Color as RatexColor;
use ratex_types::display_item::{DisplayItem, DisplayList};
use ratex_types::math_style::MathStyle;
use scal_core::Color;

use crate::anim_object::Transform;
use crate::anim_object::object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult};
use crate::anim_object::render::PipelineKind;
use crate::anim_object::svg::upload_rgba_bind_group;
use crate::renderer::{Index, Vertex};

/// The equation texture is rasterized up to this many times larger than its logical size so it
/// stays sharp when scaled or rendered at a higher resolution.
const SUPERSAMPLE: f32 = 3.0;

/// Largest texture side we allow (wgpu's default limit).
const MAX_TEXTURE_SIDE: u32 = 8192;

/// Margin around the equation in em. Superscripts, integrals and radicals can reach past the
/// layout box RaTeX reports, so the texture needs some room or their ink gets clipped.
const PAD_EM: f64 = 0.12;

/// Ink color the layout engine uses for everything the formula does not color explicitly.
/// Display items with exactly this color form the "default" layer, which follows the object's
/// color.
const DEFAULT_INK: RatexColor = RatexColor {
    r: 0.012_34,
    g: 0.567_8,
    b: 0.901_2,
    a: 1.0,
};

/// Reserved color a named part is typeset in, so its glyphs can be split into their own layer.
fn marker_color(index: usize) -> RatexColor {
    // Parts are limited to 255; the blue channel carries the index (never 0).
    RatexColor::from_hex(&format!("#0102{:02x}", index + 1)).expect("valid marker hex")
}

/// A typeset LaTeX equation. Typesetting happens once at construction. The result is kept as a
/// stack of SVG layers: symbols with a hard-coded color, one layer per named part (see
/// `var_color`) and everything else. They are rasterized lazily into one texture and drawn as
/// stacked quads, so recoloring a part only changes vertex colors.
#[derive(Clone, Debug)]
pub struct Math {
    /// The LaTeX currently shown: `template` with its placeholders replaced by `number`.
    pub latex: String,
    /// The LaTeX as written, with `{:spec}` placeholders (see `scal_core::number_format`).
    pub template: String,
    /// The number the placeholders of `template` currently show.
    pub number: f32,
    pub color: Color,
    pub font_size: f32,
    pub display: bool,
    pub transform: Transform,
    /// Current color of each named part.
    pub part_colors: Vec<(String, Color)>,
    /// Highlight state (see `MathHighlight`).
    pub highlight: MathHighlight,
    layers: std::sync::Arc<Layers>,
    size: Vec2,
    /// Hash of `latex`; tells the renderer when the cached texture is stale.
    version: u64,
}

/// Animated highlight state. Parts with weight 1 are tinted to `color`; while `active` is 1,
/// everything not highlighted is dimmed to alpha `dim`.
#[derive(Clone, Debug)]
pub struct MathHighlight {
    pub active: f32,
    pub dim: f32,
    pub color: Color,
    /// Highlight weight (0..1) per part, same order as `Math::part_colors`.
    pub weights: Vec<f32>,
}

#[derive(Debug)]
struct Layers {
    /// Symbols with a color hard-coded in the formula; `None` if there are none.
    fixed: Option<String>,
    /// One layer per named part that appears in the formula.
    parts: Vec<(String, String)>,
    /// Symbols without an explicit color.
    default: String,
}

impl Layers {
    fn count(&self) -> usize {
        usize::from(self.fixed.is_some()) + self.parts.len() + 1
    }
}

impl Math {
    pub fn new(
        latex: String,
        color: Color,
        font_size: f32,
        display: bool,
        palette: &[(String, Color)],
        transform: Transform,
    ) -> Result<Self> {
        Self::new_with_number(latex, color, font_size, display, palette, transform, 0.0)
    }

    /// Like `new`, with `number` shown by the `{:spec}` placeholders in `latex`.
    pub fn new_with_number(
        template: String,
        color: Color,
        font_size: f32,
        display: bool,
        palette: &[(String, Color)],
        transform: Transform,
        number: f32,
    ) -> Result<Self> {
        anyhow::ensure!(palette.len() < 255, "at most 254 named math parts are supported");
        let latex = scal_core::number_format::format_template(&template, number);
        let (layers, size) = typeset_cached(&latex, palette, font_size, display)?;
        Ok(Self {
            version: hash_str(&latex),
            latex,
            template,
            number,
            color,
            font_size,
            display,
            transform,
            part_colors: palette.to_vec(),
            highlight: MathHighlight {
                active: 0.0,
                dim: 1.0,
                color: Color::WHITE,
                weights: vec![0.0; palette.len()],
            },
            layers,
            size,
        })
    }

    /// Re-typesets with the placeholders showing `number`. Returns `true` if the equation
    /// changed.
    fn render_number(&mut self, number: f32) -> Result<bool> {
        self.number = number;
        let latex = scal_core::number_format::format_template(&self.template, number);
        if latex == self.latex {
            return Ok(false);
        }
        let (layers, size) =
            typeset_cached(&latex, &self.part_colors, self.font_size, self.display)?;
        self.layers = layers;
        self.size = size;
        self.version = hash_str(&latex);
        self.latex = latex;
        Ok(true)
    }
}

fn hash_str(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

type CacheKey = (String, Vec<String>, u32, bool);

/// Typesetting is the expensive part of an animated number, and the same few strings come back
/// (the same value at the same precision, scrubbing the preview, re-rendering). Keep results
/// around, keyed by everything that influences them.
fn typeset_cached(
    latex: &str,
    palette: &[(String, Color)],
    font_size: f32,
    display: bool,
) -> Result<(std::sync::Arc<Layers>, Vec2)> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    /// Entries kept before the cache is emptied.
    const MAX_ENTRIES: usize = 1024;
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, (Arc<Layers>, Vec2)>>> = OnceLock::new();

    let names: Vec<String> = palette.iter().map(|(n, _)| n.clone()).collect();
    let key = (latex.to_string(), names, font_size.to_bits(), display);
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some((layers, size)) = cache.lock().expect("math cache poisoned").get(&key) {
        return Ok((layers.clone(), *size));
    }
    let source = apply_palette(latex, palette);
    let names: Vec<&str> = palette.iter().map(|(n, _)| n.as_str()).collect();
    let (layers, size) = typeset(&source, &names, font_size, display)?;
    let layers = Arc::new(layers);
    let mut cache = cache.lock().expect("math cache poisoned");
    if cache.len() >= MAX_ENTRIES {
        cache.clear();
    }
    cache.insert(key, (layers.clone(), size));
    Ok((layers, size))
}

/// Replaces `\textcolor{name}` and `\color{name}` with the part's marker color.
fn apply_palette(latex: &str, palette: &[(String, Color)]) -> String {
    let mut out = latex.to_string();
    for (i, (name, _)) in palette.iter().enumerate() {
        let c = marker_color(i);
        let hex = format!(
            "#{:02x}{:02x}{:02x}",
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8
        );
        for cmd in ["\\textcolor", "\\color"] {
            out = out.replace(&format!("{cmd}{{{name}}}"), &format!("{cmd}{{{hex}}}"));
        }
    }
    out
}

fn item_color(item: &DisplayItem) -> &RatexColor {
    match item {
        DisplayItem::GlyphPath { color, .. }
        | DisplayItem::Line { color, .. }
        | DisplayItem::Rect { color, .. }
        | DisplayItem::Path { color, .. } => color,
    }
}

fn to_svg(list: &DisplayList, size: f64) -> String {
    let svg = render_to_svg_with_color_syntax(
        list,
        &SvgOptions {
            font_size: size,
            padding: size * PAD_EM,
            stroke_width: (size / 16.0).max(0.5),
            embed_glyphs: true,
            font_dir: String::new(),
        },
        SvgColorSyntax::Rgb,
    );
    // RaTeX writes the root width/height in `pt`; usvg would convert those to px (x4/3).
    strip_root_units(svg)
}

fn typeset(latex: &str, part_names: &[&str], font_size: f32, display: bool) -> Result<(Layers, Vec2)> {
    let ast = parse(latex.trim()).map_err(|e| match e.loc {
        Some(loc) => anyhow!("invalid LaTeX {latex:?}: {} (at byte {})", e.message, loc.start),
        None => anyhow!("invalid LaTeX {latex:?}: {}", e.message),
    })?;
    let style = if display {
        MathStyle::Display
    } else {
        MathStyle::Text
    };
    let opts = LayoutOptions::default()
        .with_style(style)
        .with_color(DEFAULT_INK);
    let list = to_display_list(&layout(&ast, &opts));
    let size = f64::from(font_size);

    // Every layer keeps the full width/height/depth so they line up when stacked.
    let markers: Vec<RatexColor> = (0..part_names.len()).map(marker_color).collect();
    let select = |pred: &dyn Fn(&RatexColor) -> bool| DisplayList {
        items: list
            .items
            .iter()
            .filter(|i| pred(item_color(i)))
            .cloned()
            .collect(),
        width: list.width,
        height: list.height,
        depth: list.depth,
    };
    let default = select(&|c| *c == DEFAULT_INK);
    let fixed = select(&|c| *c != DEFAULT_INK && !markers.contains(c));
    let parts = part_names
        .iter()
        .zip(&markers)
        .map(|(name, marker)| ((*name).to_string(), select(&|c| c == marker)))
        .filter(|(_, l)| !l.items.is_empty())
        .map(|(name, l)| (name, to_svg(&l, size)))
        .collect();

    let layers = Layers {
        fixed: (!fixed.items.is_empty()).then(|| to_svg(&fixed, size)),
        parts,
        default: to_svg(&default, size),
    };
    let pad = size * PAD_EM * 2.0;
    let w = ((list.width * size + pad) as f32).max(1.0);
    let h = (((list.height + list.depth) * size + pad) as f32).max(1.0);
    Ok((layers, Vec2::new(w, h)))
}

fn strip_root_units(svg: String) -> String {
    let Some(head_end) = svg.find('>') else {
        return svg;
    };
    let (head, rest) = svg.split_at(head_end);
    let mut head = head.to_string();
    for attr in ["width=\"", "height=\""] {
        if let Some(i) = head.find(attr) {
            let v = i + attr.len();
            if let Some(q) = head[v..].find('"') {
                let value = head[v..v + q].trim_end_matches("pt").to_string();
                head.replace_range(v..v + q, &value);
            }
        }
    }
    head.push_str(rest);
    head
}

fn render_layer(svg: &str, w: u32, h: u32) -> Result<tiny_skia::Pixmap> {
    let tree = usvg::Tree::from_data(svg.as_bytes(), &usvg::Options::default())
        .context("while parsing the typeset equation")?;
    let mut pixmap =
        tiny_skia::Pixmap::new(w, h).ok_or_else(|| anyhow!("failed to create pixmap {w}x{h}"))?;
    let tsize = tree.size();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(w as f32 / tsize.width(), h as f32 / tsize.height()),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Appends the layer as white ink; only coverage (alpha) matters.
fn push_mask(pixels: &mut Vec<u8>, pixmap: &tiny_skia::Pixmap) {
    for px in pixmap.pixels() {
        pixels.extend([255, 255, 255, px.alpha()]);
    }
}

/// Rasterizes the layers into one RGBA texture, stacked vertically in this order: hard-coded
/// colors (straight alpha, own colors), each named part (white mask), then the default layer
/// (white mask). Returns the pixels and the texture size.
fn rasterize(layers: &Layers, size: Vec2) -> Result<(Vec<u8>, u32, u32)> {
    let n = layers.count() as f32;
    // Shrink the supersampling so the whole stack fits in one texture.
    let ss = SUPERSAMPLE
        .min(MAX_TEXTURE_SIDE as f32 / (size.y * n))
        .min(MAX_TEXTURE_SIDE as f32 / size.x);
    anyhow::ensure!(ss >= 0.5, "equation is too large to rasterize ({size:?}, {n} layers)");
    let w = ((size.x * ss).ceil() as u32).max(1);
    let h = ((size.y * ss).ceil() as u32).max(1);

    let mut pixels = Vec::new();
    if let Some(fixed) = &layers.fixed {
        for px in render_layer(fixed, w, h)?.pixels() {
            let c = px.demultiply();
            pixels.extend([c.red(), c.green(), c.blue(), c.alpha()]);
        }
    }
    for (_, svg) in &layers.parts {
        push_mask(&mut pixels, &render_layer(svg, w, h)?);
    }
    push_mask(&mut pixels, &render_layer(&layers.default, w, h)?);
    Ok((pixels, w, h * layers.count() as u32))
}

impl AnimObjectTrait for Math {
    fn number(&self) -> Option<f32> {
        scal_core::number_format::has_placeholder(&self.template).then_some(self.number)
    }
    fn set_number(&mut self, number: f32) -> Result<bool> {
        anyhow::ensure!(
            scal_core::number_format::has_placeholder(&self.template),
            "equation {:?} has no {{:spec}} placeholder to show a number in",
            self.template
        );
        self.render_number(number)
    }
    fn content_version(&self) -> u64 {
        self.version
    }
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        self.size
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        let half = self.size * 0.5;
        let fade = self.color.a;
        let hl = &self.highlight;
        // Alpha multiplier for a layer with highlight weight `w`.
        let dim_for = |w: f32| 1.0 - hl.active * (1.0 - hl.dim) * (1.0 - w);
        // Vertex color of each stacked layer, in texture order. Hard-coded colors are not
        // tinted but fade with the object's alpha; parts use their own current color.
        let mut colors: Vec<Color> = Vec::new();
        if self.layers.fixed.is_some() {
            colors.push(Color::new(1.0, 1.0, 1.0, fade * dim_for(0.0)));
        }
        for (name, _) in &self.layers.parts {
            let idx = self.part_colors.iter().position(|(n, _)| n == name);
            let base = idx.map_or(Color::WHITE, |i| self.part_colors[i].1);
            let w = idx.map_or(0.0, |i| hl.weights[i]);
            let mix = |a: f32, b: f32| a + w * (b - a);
            colors.push(Color::new(
                mix(base.r, hl.color.r),
                mix(base.g, hl.color.g),
                mix(base.b, hl.color.b),
                base.a * fade * dim_for(w),
            ));
        }
        colors.push(Color::new(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a * dim_for(0.0),
        ));

        let n = colors.len() as f32;
        let mut vertices = Vec::new();
        let mut indices: Vec<Index> = Vec::new();
        for (i, color) in colors.into_iter().enumerate() {
            let (v0, v1) = (i as f32 / n, (i + 1) as f32 / n);
            let base = vertices.len() as Index;
            let v = |position: Vec2, uv: Vec2| Vertex {
                position,
                color,
                uv,
            };
            vertices.extend([
                v(-half, Vec2::new(0.0, v0)),
                v(Vec2::new(half.x, -half.y), Vec2::new(1.0, v0)),
                v(half, Vec2::new(1.0, v1)),
                v(Vec2::new(-half.x, half.y), Vec2::new(0.0, v1)),
            ]);
            indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
        }
        Ok((vertices, indices, PipelineKind::Image))
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        let layers = self.layers.clone();
        let size = self.size;
        let label = format!("math_{}", self.latex);
        Some(Box::new(move |device, queue| {
            match rasterize(&layers, size) {
                Ok((pixels, w, h)) => {
                    vec![upload_rgba_bind_group(device, queue, &label, &pixels, w, h)]
                }
                Err(e) => {
                    log::error!("Failed to rasterize equation: {e:?}");
                    vec![]
                }
            }
        }))
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r"\frac{a}{b} + \sqrt{x^2} = \sum_{i=0}^n i";

    fn ink(pixels: &[u8], w: usize, row: std::ops::Range<usize>) -> Vec<(usize, usize, [u8; 4])> {
        let mut v = Vec::new();
        for y in row {
            for x in 0..w {
                let i = (y * w + x) * 4;
                if pixels[i + 3] > 200 {
                    v.push((x, y, [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]));
                }
            }
        }
        v
    }

    #[test]
    fn typesets_and_rasterizes() {
        let (layers, size) = typeset(SRC, &[], 32.0, true).unwrap();
        assert!(size.x > 20.0 && size.y > 20.0, "{size:?}");
        assert!(layers.fixed.is_none() && layers.parts.is_empty());
        let (pixels, w, h) = rasterize(&layers, size).unwrap();
        assert_eq!(pixels.len(), (w * h * 4) as usize);
        assert!(pixels.chunks_exact(4).any(|p| p[3] > 0), "nothing was drawn");
    }

    #[test]
    fn reports_invalid_latex() {
        assert!(typeset(r"\frac{a", &[], 32.0, true).is_err());
    }

    #[test]
    fn palette_names_become_distinct_markers() {
        let palette = [("a".to_string(), Color::RED), ("b".to_string(), Color::RED)];
        let out = apply_palette(r"\textcolor{a}{x} \color{b} \textcolor{c}{y}", &palette);
        assert_eq!(out, r"\textcolor{#010201}{x} \color{#010202} \textcolor{c}{y}");
    }

    #[test]
    fn each_part_gets_its_own_layer_even_with_equal_colors() {
        let palette = [("a".to_string(), Color::RED), ("b".to_string(), Color::RED)];
        let src = apply_palette(r"\textcolor{a}{p} + \textcolor{b}{q} + \textcolor{#00ff00}{r} + s", &palette);
        let (layers, size) = typeset(&src, &["a", "b"], 48.0, true).unwrap();
        assert_eq!(layers.parts.len(), 2);
        assert!(layers.fixed.is_some());
        assert_eq!(layers.count(), 4);
        let (pixels, w, h) = rasterize(&layers, size).unwrap();
        let (w, rows) = (w as usize, h as usize / 4);
        // fixed layer (first): the hard-coded green keeps its color.
        let fixed = ink(&pixels, w, 0..rows);
        assert!(!fixed.is_empty() && fixed.iter().all(|(_, _, c)| c[1] > 200 && c[0] < 50));
        // parts and default are white masks with ink, at different horizontal positions.
        let a = ink(&pixels, w, rows..2 * rows);
        let b = ink(&pixels, w, 2 * rows..3 * rows);
        let d = ink(&pixels, w, 3 * rows..4 * rows);
        for layer in [&a, &b, &d] {
            assert!(!layer.is_empty());
            assert!(layer.iter().all(|(_, _, c)| c[..3] == [255, 255, 255]));
        }
        let max_x = |l: &Vec<(usize, usize, [u8; 4])>| l.iter().map(|p| p.0).max().unwrap();
        let min_x = |l: &Vec<(usize, usize, [u8; 4])>| l.iter().map(|p| p.0).min().unwrap();
        assert!(max_x(&a) < min_x(&b), "p must be left of q");
    }

    #[test]
    fn part_color_only_changes_vertex_colors() {
        let palette = vec![("a".to_string(), Color::RED)];
        let mut m = Math::new(
            r"\textcolor{a}{x} + y".into(),
            Color::WHITE,
            32.0,
            true,
            &palette,
            Transform::new(None, glam::Vec3::ZERO, 0.0, glam::Vec2::ONE),
        )
        .unwrap();
        let mut mgr = crate::anim_object::text::TextManager::new(1.0);
        let (v1, i1, _) = m.generate_mesh(&mut mgr).unwrap();
        m.part_colors[0].1 = Color::GREEN;
        let (v2, i2, _) = m.generate_mesh(&mut mgr).unwrap();
        assert_eq!(i1, i2);
        assert_eq!(v1.len(), 8);
        // quad 0 is part `a`, quad 1 the default layer
        assert_eq!(v1[0].color, Color::RED);
        assert_eq!(v2[0].color, Color::GREEN);
        assert_eq!(v1[4].color, v2[4].color);
        assert_eq!(v1[0].position, v2[0].position);
    }

    #[test]
    fn highlight_tints_the_part_and_dims_the_rest() {
        let palette = vec![("a".to_string(), Color::RED), ("b".to_string(), Color::BLUE)];
        let mut m = Math::new(
            r"\textcolor{a}{x} + \textcolor{b}{z} + y".into(),
            Color::WHITE,
            32.0,
            true,
            &palette,
            Transform::new(None, glam::Vec3::ZERO, 0.0, glam::Vec2::ONE),
        )
        .unwrap();
        let mut mgr = crate::anim_object::text::TextManager::new(1.0);
        m.highlight = MathHighlight {
            active: 1.0,
            dim: 0.2,
            color: Color::GREEN,
            weights: vec![1.0, 0.0],
        };
        let (v, _, _) = m.generate_mesh(&mut mgr).unwrap();
        // quads: a, b, default
        assert_eq!(v[0].color, Color::GREEN); // highlighted: tinted, fully opaque
        assert_eq!(v[4].color.b, 1.0); // b keeps its color...
        assert!((v[4].color.a - 0.2).abs() < 1e-6); // ...but is dimmed
        assert!((v[8].color.a - 0.2).abs() < 1e-6); // default ink dimmed
        m.highlight.active = 0.0;
        let (v, _, _) = m.generate_mesh(&mut mgr).unwrap();
        assert_eq!(v[4].color.a, 1.0);
        assert_eq!(v[8].color.a, 1.0);
    }

    #[test]
    fn ink_stays_inside_the_texture() {
        // Superscripts can reach above the layout box; the outermost rows/columns must be empty.
        for src in [r"ax^2 + bx + c = 0", r"\int_0^1 x^2\,dx", r"\sqrt{b^2-4ac}"] {
            let (layers, size) = typeset(src, &[], 110.0, true).unwrap();
            let (pixels, w, h) = rasterize(&layers, size).unwrap();
            let (w, h) = (w as usize, h as usize);
            let a = |x: usize, y: usize| pixels[(y * w + x) * 4 + 3];
            assert!((0..w).all(|x| a(x, 0) == 0), "ink touches top edge for {src}");
            assert!((0..w).all(|x| a(x, h - 1) == 0), "ink touches bottom edge for {src}");
            assert!((0..h).all(|y| a(0, y) == 0), "ink touches left edge for {src}");
            assert!((0..h).all(|y| a(w - 1, y) == 0), "ink touches right edge for {src}");
        }
    }

    #[test]
    fn many_parts_fit_in_a_texture() {
        let names: Vec<String> = (0..12).map(|i| format!("p{i}")).collect();
        let palette: Vec<(String, Color)> = names.iter().map(|n| (n.clone(), Color::RED)).collect();
        let latex: String = names.iter().map(|n| format!(r"\textcolor{{{n}}}{{x}}+")).collect::<String>() + "1";
        let m = Math::new(latex, Color::WHITE, 200.0, true, &palette, Transform::new(None, glam::Vec3::ZERO, 0.0, glam::Vec2::ONE)).unwrap();
        let (_, w, h) = rasterize(&m.layers, m.size).unwrap();
        assert!(w <= MAX_TEXTURE_SIDE && h <= MAX_TEXTURE_SIDE, "{w}x{h}");
    }
}
