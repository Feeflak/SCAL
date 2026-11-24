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

/// The equation texture is rasterized this many times larger than its logical size so it stays
/// sharp when scaled or rendered at a higher resolution.
const SUPERSAMPLE: f32 = 3.0;

/// Margin around the equation in em. Superscripts, integrals and radicals can reach past the
/// layout box RaTeX reports, so the texture needs some room or their ink gets clipped.
const PAD_EM: f64 = 0.12;

/// Ink color the layout engine uses for everything the formula does not color explicitly.
/// Display items with exactly this color form the "default" layer, which follows the object's
/// color. Everything else keeps the color given in the formula.
const DEFAULT_INK: RatexColor = RatexColor {
    r: 0.012_34,
    g: 0.567_8,
    b: 0.901_2,
    a: 1.0,
};

/// A typeset LaTeX equation. Typesetting happens once at construction. The result is kept as two
/// SVGs: symbols with an explicit color (`\textcolor`) and everything else (white ink, tinted by
/// `color`). They are rasterized lazily into one texture and drawn as two stacked quads.
#[derive(Clone, Debug)]
pub struct Math {
    pub latex: String,
    pub color: Color,
    pub font_size: f32,
    pub display: bool,
    pub transform: Transform,
    layers: std::sync::Arc<Layers>,
    size: Vec2,
}

#[derive(Debug)]
struct Layers {
    /// Symbols with a color set in the formula; `None` if there are none.
    fixed: Option<String>,
    /// Symbols without an explicit color.
    default: String,
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
        let source = apply_palette(&latex, palette);
        let (layers, size) = typeset(&source, font_size, display)?;
        Ok(Self {
            latex,
            color,
            font_size,
            display,
            transform,
            layers: std::sync::Arc::new(layers),
            size,
        })
    }
}

/// Replaces `\textcolor{name}` and `\color{name}` with the palette color as `#rrggbb`.
fn apply_palette(latex: &str, palette: &[(String, Color)]) -> String {
    let mut out = latex.to_string();
    for (name, color) in palette {
        let hex = scal_core::latex_color(*color);
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

fn typeset(latex: &str, font_size: f32, display: bool) -> Result<(Layers, Vec2)> {
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

    // Both layers keep the full width/height/depth so they line up when stacked.
    let split = |keep_default: bool| DisplayList {
        items: list
            .items
            .iter()
            .filter(|i| (*item_color(i) == DEFAULT_INK) == keep_default)
            .cloned()
            .collect(),
        width: list.width,
        height: list.height,
        depth: list.depth,
    };
    let default = split(true);
    let fixed = split(false);

    let layers = Layers {
        fixed: (!fixed.items.is_empty()).then(|| to_svg(&fixed, size)),
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

/// Rasterizes the layers into one RGBA texture. With explicitly colored symbols the texture is
/// two stacked halves: the fixed-color layer on top (straight alpha, own colors) and the default
/// layer below (white, alpha only). Without them it is just the default layer.
fn rasterize(layers: &Layers, size: Vec2) -> Result<(Vec<u8>, u32, u32)> {
    let w = ((size.x * SUPERSAMPLE).ceil() as u32).max(1);
    let h = ((size.y * SUPERSAMPLE).ceil() as u32).max(1);

    let mut pixels = Vec::new();
    if let Some(fixed) = &layers.fixed {
        let pixmap = render_layer(fixed, w, h)?;
        for px in pixmap.pixels() {
            let c = px.demultiply();
            pixels.extend([c.red(), c.green(), c.blue(), c.alpha()]);
        }
    }
    let default = render_layer(&layers.default, w, h)?;
    // The ink is white; only coverage (alpha) matters.
    for px in default.pixels() {
        pixels.extend([255, 255, 255, px.alpha()]);
    }
    let rows = if layers.fixed.is_some() { h * 2 } else { h };
    Ok((pixels, w, rows))
}

impl AnimObjectTrait for Math {
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
        let two = self.layers.fixed.is_some();
        // (v range in the texture, vertex color) per quad, drawn in order.
        let mut quads: Vec<((f32, f32), Color)> = Vec::new();
        if two {
            // Fixed colors are not tinted, but still fade with the object's alpha.
            let fade = Color::new(1.0, 1.0, 1.0, self.color.a);
            quads.push(((0.0, 0.5), fade));
            quads.push(((0.5, 1.0), self.color));
        } else {
            quads.push(((0.0, 1.0), self.color));
        }
        let mut vertices = Vec::new();
        let mut indices: Vec<Index> = Vec::new();
        for ((v0, v1), color) in quads {
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

    #[test]
    fn typesets_and_rasterizes() {
        let (layers, size) = typeset(SRC, 32.0, true).unwrap();
        assert!(size.x > 20.0 && size.y > 20.0, "{size:?}");
        assert!(layers.fixed.is_none());
        let (pixels, w, h) = rasterize(&layers, size).unwrap();
        assert_eq!(pixels.len(), (w * h * 4) as usize);
        assert!(pixels.chunks_exact(4).any(|p| p[3] > 0), "nothing was drawn");
    }

    #[test]
    fn reports_invalid_latex() {
        assert!(typeset(r"\frac{a", 32.0, true).is_err());
    }

    #[test]
    fn palette_names_are_substituted() {
        let palette = [("a".to_string(), Color::RED)];
        assert_eq!(
            apply_palette(r"\textcolor{a}{x} \color{a} \textcolor{b}{y}", &palette),
            r"\textcolor{#ff0000}{x} \color{#ff0000} \textcolor{b}{y}"
        );
    }

    #[test]
    fn explicit_colors_form_their_own_layer() {
        let src = apply_palette(
            r"\textcolor{r}{a} + \textcolor{g}{b} = c",
            &[("r".into(), Color::RED), ("g".into(), Color::GREEN)],
        );
        let (layers, size) = typeset(&src, 48.0, true).unwrap();
        assert!(layers.fixed.is_some());
        let (pixels, w, h) = rasterize(&layers, size).unwrap();
        assert_eq!(h % 2, 0);
        assert_eq!(pixels.len(), (w * h * 4) as usize);
        let (top, bottom) = pixels.split_at(pixels.len() / 2);
        let red = top.chunks_exact(4).any(|p| p[3] > 200 && p[0] > 200 && p[1] < 50);
        let green = top.chunks_exact(4).any(|p| p[3] > 200 && p[1] > 200 && p[0] < 50);
        assert!(red && green, "expected both colors in the fixed layer");
        // `+`, `=` and `c` are default ink: present in the bottom half, always white.
        assert!(bottom.chunks_exact(4).any(|p| p[3] > 0));
        assert!(bottom.chunks_exact(4).all(|p| p[..3] == [255, 255, 255]));
    }

    #[test]
    fn ink_stays_inside_the_texture() {
        // Superscripts can reach above the layout box; the outermost rows/columns must be empty.
        for src in [r"ax^2 + bx + c = 0", r"\int_0^1 x^2\,dx", r"\sqrt{b^2-4ac}"] {
            let (layers, size) = typeset(src, 110.0, true).unwrap();
            let (pixels, w, h) = rasterize(&layers, size).unwrap();
            let (w, h) = (w as usize, h as usize);
            let a = |x: usize, y: usize| pixels[(y * w + x) * 4 + 3];
            assert!((0..w).all(|x| a(x, 0) == 0), "ink touches top edge for {src}");
            assert!((0..w).all(|x| a(x, h - 1) == 0), "ink touches bottom edge for {src}");
            assert!((0..h).all(|y| a(0, y) == 0), "ink touches left edge for {src}");
            assert!((0..h).all(|y| a(w - 1, y) == 0), "ink touches right edge for {src}");
        }
    }
}
