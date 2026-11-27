use glam::{Vec2, Vec3};
use uuid::Uuid;

use crate::anim_obj::DEFAULT_HIGHLIGHT;
use crate::anim_obj::{
    Alignment, AnimObj, AnimObjKind, CodeHandle, CodeWindowHandle, LayoutDir, MathHandle, ScrollLayoutHandle,
    StretchMode, Syntax, TerminalHandle, TextModifier, TextSpan, VectorHandle,
};
use crate::color::Color;
use crate::motion::IntoScale;
use crate::theme::Theme;
use crate::transform::Transform;

macro_rules! impl_transform_methods {
    ($builder:ty) => {
        #[allow(clippy::return_self_not_must_use)]
        impl $builder {
            /// Set the 2D position of this object's transform
            pub const fn pos(mut self, position: Vec2) -> Self {
                self.transform.position = position.extend(self.transform.position.z);
                self
            }

            /// Set the z position of this object's transform. It's used for ordering object drawing.
            pub const fn z(mut self, z: f32) -> Self {
                self.transform.position.z = z;
                self
            }
            /// Set the scale of this object's transform. Takes a `Vec2` or an `f32` for uniform
            /// scale: `.scale(0.5)`.
            pub fn scale(mut self, scale: impl IntoScale) -> Self {
                self.transform.scale = scale.into_scale();
                self
            }
            /// Start invisible (scale zero). Bring it in later with `pop_in()` or
            /// `scale_to(..)`; it can already be instantiated while hidden.
            pub const fn hidden(mut self) -> Self {
                self.transform.scale = Vec2::ZERO;
                self
            }
            /// Set the rotation of this object's transform
            pub const fn rot(mut self, rotation: f32) -> Self {
                self.transform.rotation = rotation;
                self
            }
            /// Set the point this object rotates around, as an offset in pixels from its center
            /// (y points down). The default is the center. This does not move the object.
            /// ```ignore
            /// // rotate a 400px wide wing around its quarter chord
            /// svg().path("wing.svg").size(vec2(400.0, 80.0)).pivot(vec2(-100.0, 0.0))
            /// ```
            pub const fn pivot(mut self, pivot: Vec2) -> Self {
                self.transform.pivot = pivot;
                self
            }
        }
    };
}

/// Create a new SVG object builder.
/// ```ignore
/// svg()
///     .path("./icon.svg")
///     .size(Vec2::splat(60.0))
///     .color(Color::RED)
///     .build(),
/// ```
pub fn svg() -> SvgBuilder {
    SvgBuilder::default()
}

#[must_use]
pub struct SvgBuilder {
    path: String,
    size: Vec2,
    tint: Color,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: Option<f32>,
    stretch: StretchMode,
    transform: Transform,
}

impl Default for SvgBuilder {
    fn default() -> Self {
        Self {
            path: String::new(),
            size: Vec2::splat(40.0),
            tint: Color::WHITE,
            fill: None,
            stroke: None,
            stroke_width: None,
            stretch: StretchMode::Fit,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl SvgBuilder {
    /// Set the filepath of the SVG
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    /// Set the size of the SVG
    pub const fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    /// Set the tint color of the SVG
    pub const fn color(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }
    /// Set the fill color of the SVG
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    /// Set the stroke color of the SVG
    pub const fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
    /// Set the stroke width of the SVG
    pub const fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }
    /// Set the stretch mode of the SVG
    pub const fn stretch(mut self, stretch: StretchMode) -> Self {
        self.stretch = stretch;
        self
    }
    #[must_use]
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Svg {
                path: self.path,
                size: self.size,
                tint: self.tint,
                fill: self.fill,
                stroke: self.stroke,
                stroke_width: self.stroke_width,
                stretch: self.stretch,
            },
        }
    }
}

impl_transform_methods!(SvgBuilder);

/// Create a new rectangle object builder.
/// ```ignore
/// rectangle()
///     .size(Vec2::new(200.0, 100.0))
///     .corner_radius(10.0)
///     .color(Color::BLUE)
///     .build(),
/// ```
pub fn rectangle() -> RectangleBuilder {
    RectangleBuilder::default()
}

#[must_use]
pub struct RectangleBuilder {
    size: Vec2,
    corner_radius: f32,
    color: Color,
    transform: Transform,
}

impl Default for RectangleBuilder {
    fn default() -> Self {
        Self {
            size: Vec2::splat(100.0),
            corner_radius: 0.0,
            color: Color::WHITE,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl RectangleBuilder {
    pub const fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub const fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    #[must_use]
    pub const fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Rectangle {
                size: self.size,
                corner_radius: self.corner_radius,
                color: self.color,
            },
        }
    }
}

impl_transform_methods!(RectangleBuilder);

/// Create a new circle object builder.
/// ```ignore
/// circle()
///     .radius(75.0)
///     .color(Color::GREEN)
///     .build(),
/// ```
pub fn circle() -> CircleBuilder {
    CircleBuilder::default()
}

#[must_use]
pub struct CircleBuilder {
    radius: f32,
    color: Color,
    transform: Transform,
}

impl Default for CircleBuilder {
    fn default() -> Self {
        Self {
            radius: 50.0,
            color: Color::WHITE,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl CircleBuilder {
    /// Set the radius of the circle
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    /// Set the color of the circle
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    #[must_use]
    pub const fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Circle {
                radius: self.radius,
                color: self.color,
            },
        }
    }
}

impl_transform_methods!(CircleBuilder);

/// Create a new code block object builder.
/// ```ignore
/// code()
///     .source("fn main() {\n    println!(\"hello\");\n}")
///     .syntax(Syntax::Rust)
///     .line_numbers(true)
///     .build(),
/// ```
pub fn code() -> CodeBuilder {
    CodeBuilder::default()
}

#[must_use]
pub struct CodeBuilder {
    source_code: String,
    font_family: String,
    font_size: f32,
    syntax: Syntax,
    theme: Option<Theme>,
    padding: f32,
    show_line_numbers: bool,
    line_number_color: Color,
    transform: Transform,
}

impl Default for CodeBuilder {
    fn default() -> Self {
        Self {
            source_code: String::new(),
            font_family: "sans-serif".to_string(),
            font_size: 20.0,
            syntax: Syntax::Rust,
            theme: None,
            padding: 20.0,
            show_line_numbers: false,
            line_number_color: Color::new(0.5, 0.5, 0.5, 0.6),
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl CodeBuilder {
    /// Set the source code of the code block
    pub fn source(mut self, code: impl Into<String>) -> Self {
        self.source_code = code.into();
        self
    }
    /// Set the font family of the code block
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the font size of the code block
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Set the syntax highlighting language of the code block
    pub const fn syntax(mut self, syntax: Syntax) -> Self {
        self.syntax = syntax;
        self
    }
    /// Set the theme of the code block
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
    /// Set the padding of the code block
    pub const fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    /// Toggle line numbers in the code block
    pub const fn line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
    #[must_use]
    pub fn build(self) -> CodeHandle {
        let id = self.transform.uuid;
        CodeHandle(AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Code {
                source_code: self.source_code,
                font_family: self.font_family,
                font_size: self.font_size,
                syntax: self.syntax,
                theme: self.theme,
                padding: self.padding,
                show_line_numbers: self.show_line_numbers,
                line_number_color: self.line_number_color,
            },
        })
    }
}

impl_transform_methods!(CodeBuilder);

/// Create a new polygon object builder.
/// ```ignore
/// polygon()
///     .radius(60.0)
///     .sides(8)
///     .color(Color::YELLOW)
///     .build(),
/// ```
pub fn polygon() -> PolygonBuilder {
    PolygonBuilder::default()
}

#[must_use]
pub struct PolygonBuilder {
    radius: f32,
    sides: u32,
    color: Color,
    transform: Transform,
}

impl Default for PolygonBuilder {
    fn default() -> Self {
        Self {
            radius: 50.0,
            sides: 6,
            color: Color::WHITE,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

impl PolygonBuilder {
    /// Set the radius of the polygon
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    /// Set the number of sides of the polygon
    pub const fn sides(mut self, sides: u32) -> Self {
        self.sides = sides;
        self
    }
    /// Set the color of the polygon
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    #[must_use]
    pub const fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Polygon {
                radius: self.radius,
                sides: self.sides,
                color: self.color,
            },
        }
    }
}

impl_transform_methods!(PolygonBuilder);

/// Create a new vector builder: a line through some points, optionally ending in an arrow head.
///
/// Points are scene coordinates (like `.pos(..)` of other objects). The arrow head points the
/// way the line goes from the second to last point to the last one. Animate the points with
/// `move_point`, `move_points` and `points_to` on the handle.
/// ```ignore
/// let force = vector()
///     .points([vec2(400., 800.), vec2(900., 500.)])
///     .color(Color::RED)
///     .thickness(8.0)
///     .build();
///
/// // dashed line through three points, without an arrow head
/// let path = vector()
///     .points([vec2(0., 0.), vec2(300., 200.), vec2(600., 0.)])
///     .arrow_head(false)
///     .dashed(true)
///     .build();
///
/// force.move_point(1, vec2(900., 200.)).over(1.s()).ease(Ease::InOutCubic)
/// ```
pub fn vector() -> VectorBuilder {
    VectorBuilder::default()
}

#[must_use]
pub struct VectorBuilder {
    points: Vec<Vec2>,
    color: Color,
    thickness: f32,
    arrow_head: bool,
    head_length: Option<f32>,
    head_width: Option<f32>,
    dash: Option<(f32, f32)>,
    dashed: bool,
    transform: Transform,
}

impl Default for VectorBuilder {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            color: Color::WHITE,
            thickness: 6.0,
            arrow_head: true,
            head_length: None,
            head_width: None,
            dash: None,
            dashed: false,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl VectorBuilder {
    /// Set all the points the line goes through, in order. Needs at least two.
    pub fn points<P: Into<Vec2>>(mut self, points: impl IntoIterator<Item = P>) -> Self {
        self.points = points.into_iter().map(Into::into).collect();
        self
    }
    /// Add one more point at the end of the line
    pub fn point(mut self, point: impl Into<Vec2>) -> Self {
        self.points.push(point.into());
        self
    }
    /// Set the color of the line and the arrow head
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the thickness of the line in pixels (default 6)
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }
    /// Draw an arrow head on the last point (default on)
    pub const fn arrow_head(mut self, arrow_head: bool) -> Self {
        self.arrow_head = arrow_head;
        self
    }
    /// Set the length of the arrow head, from its base to the tip (default 5x the thickness).
    /// It shrinks when the last segment of the line gets shorter than this.
    pub const fn head_length(mut self, length: f32) -> Self {
        self.head_length = Some(length);
        self
    }
    /// Set the width of the arrow head (default 80% of its length)
    pub const fn head_width(mut self, width: f32) -> Self {
        self.head_width = Some(width);
        self
    }
    /// Draw the line dashed (default off). The dashes are 3x the thickness long with a gap of
    /// 2x the thickness, use [`VectorBuilder::dash`] to change that.
    pub const fn dashed(mut self, dashed: bool) -> Self {
        self.dashed = dashed;
        self
    }
    /// Draw the line dashed, with dashes of `length` and gaps of `gap` pixels
    pub const fn dash(mut self, length: f32, gap: f32) -> Self {
        self.dashed = true;
        self.dash = Some((length, gap));
        self
    }
    /// Set the z position of this object's transform. It's used for ordering object drawing.
    pub const fn z(mut self, z: f32) -> Self {
        self.transform.position.z = z;
        self
    }
    /// Set the scale of this object's transform, around the middle of the line. Takes a `Vec2`
    /// or an `f32` for uniform scale.
    pub fn scale(mut self, scale: impl IntoScale) -> Self {
        self.transform.scale = scale.into_scale();
        self
    }
    /// Start invisible (scale zero). Bring it in later with `pop_in()` or `scale_to(..)`.
    pub const fn hidden(mut self) -> Self {
        self.transform.scale = Vec2::ZERO;
        self
    }
    /// Set the rotation of this object's transform, around the middle of the line
    pub const fn rot(mut self, rotation: f32) -> Self {
        self.transform.rotation = rotation;
        self
    }

    /// # Panics
    /// If there are fewer than two points.
    #[must_use]
    pub fn build(mut self) -> VectorHandle {
        assert!(
            self.points.len() >= 2,
            "vector() needs at least two points, got {}",
            self.points.len()
        );
        // The object sits in the middle of its points, so it scales and rotates around it
        // like every other object. The points are stored relative to that middle.
        let min = self.points.iter().copied().fold(Vec2::splat(f32::MAX), Vec2::min);
        let max = self.points.iter().copied().fold(Vec2::splat(f32::MIN), Vec2::max);
        let center = (min + max) * 0.5;
        self.transform.position.x = center.x;
        self.transform.position.y = center.y;

        let head_length = self.head_length.unwrap_or(self.thickness * 5.0);
        let head_width = self.head_width.unwrap_or(head_length * 0.8);
        let dash = self.dashed.then(|| {
            self.dash
                .unwrap_or((self.thickness * 3.0, self.thickness * 2.0))
        });
        let id = self.transform.uuid;
        VectorHandle(AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Vector {
                points: self.points.iter().map(|p| *p - center).collect(),
                color: self.color,
                thickness: self.thickness,
                arrow_head: self.arrow_head,
                head_length,
                head_width,
                dash,
            },
        })
    }
}

/// Create a new image object builder.
/// ```ignore
/// image()
///     .path("./photo.png")
///     .size(Vec2::new(320.0, 240.0))
///     .build(),
/// ```
pub fn image() -> ImageBuilder {
    ImageBuilder::default()
}

#[must_use]
pub struct ImageBuilder {
    path: String,
    size: Vec2,
    color: Color,
    stretch: StretchMode,
    transform: Transform,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self {
            path: String::new(),
            size: Vec2::splat(100.0),
            color: Color::WHITE,
            stretch: StretchMode::Fit,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

impl ImageBuilder {
    /// Set the filepath of the image
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    /// Set the size of the image
    pub const fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    /// Set the color of the image
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the stretch mode of the image
    pub const fn stretch(mut self, stretch: StretchMode) -> Self {
        self.stretch = stretch;
        self
    }
    #[must_use]
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Image {
                path: self.path,
                size: self.size,
                color: self.color,
                stretch: self.stretch,
            },
        }
    }
}

impl_transform_methods!(ImageBuilder);

/// Create a new text object builder.
/// ```ignore
/// text()
///     .value("Hello, SCAL!")
///     .font_size(32.0)
///     .color(Color::WHITE)
///     .build(),
/// ```
pub fn text() -> TextBuilder {
    TextBuilder::default()
}

#[must_use]
pub struct TextBuilder {
    value: String,
    font_family: String,
    align: Alignment,
    color: Color,
    font_size: f32,
    transform: Transform,
    modifications: Vec<TextModifier>,
    spans: Vec<TextSpan>,
    number: f32,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self {
            value: String::new(),
            font_family: "sans-serif".to_string(),
            align: Alignment::Center,
            color: Color::WHITE,
            font_size: 24.0,
            transform: Transform::new(Vec3::ZERO),
            modifications: Vec::new(),
            spans: Vec::new(),
            number: 0.0,
        }
    }
}

impl TextBuilder {
    /// Set the text content
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    /// Set the text as a list of styled sections (color, bold). Replaces `value`.
    /// ```ignore
    /// text().spans([
    ///     span("Hello "),
    ///     span("world").color(Color::RED).bold(),
    /// ])
    /// ```
    pub fn spans<I>(mut self, spans: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TextSpan>,
    {
        self.spans = spans.into_iter().map(Into::into).collect();
        self.value = self.spans.iter().map(|s| s.text.as_str()).collect();
        self
    }
    /// Set the font family of the text
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the alignment of the text
    pub const fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// Set the color of the text
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the font size of the text
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Set the number shown by `{:spec}` placeholders in the text, e.g. `{:.1}` (default 0).
    /// Animate it with [`AnimObj::number`]. See [`crate::number_format`].
    /// ```ignore
    /// text().value("AoA: {:.1}°").number(5.0).build()
    /// ```
    pub const fn number(mut self, number: f32) -> Self {
        self.number = number;
        self
    }
    /// Add a text modifier (shadow, outline, infill effect)
    pub fn modifier(mut self, modifier: TextModifier) -> Self {
        self.modifications.push(modifier);
        self
    }
    #[must_use]
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Text {
                value: self.value,
                font_family: self.font_family,
                align: self.align,
                color: self.color,
                font_size: self.font_size,
                modifications: self.modifications,
                spans: self.spans,
                number: self.number,
            },
        }
    }
}

impl_transform_methods!(TextBuilder);

/// Create a styled text section for [`TextBuilder::spans`].
pub fn span(text: impl Into<String>) -> SpanBuilder {
    SpanBuilder(TextSpan {
        text: text.into(),
        color: None,
        bold: false,
    })
}

/// Builder for a [`TextSpan`].
#[must_use]
pub struct SpanBuilder(TextSpan);

#[allow(clippy::return_self_not_must_use)]
impl SpanBuilder {
    /// Set the color of this section.
    pub const fn color(mut self, color: Color) -> Self {
        self.0.color = Some(color);
        self
    }
    /// Draw this section bold.
    pub const fn bold(mut self) -> Self {
        self.0.bold = true;
        self
    }
}

impl From<SpanBuilder> for TextSpan {
    fn from(b: SpanBuilder) -> Self {
        b.0
    }
}

impl From<&str> for TextSpan {
    fn from(s: &str) -> Self {
        Self {
            text: s.to_string(),
            color: None,
            bold: false,
        }
    }
}

impl From<String> for TextSpan {
    fn from(s: String) -> Self {
        Self {
            text: s,
            color: None,
            bold: false,
        }
    }
}

/// Create a new math equation object builder. Takes LaTeX math syntax.
/// ```ignore
/// math(r"\int_0^1 x^2\,dx = \frac{1}{3}")
///     .font_size(48.0)
///     .color(Color::WHITE)
///     .build(),
/// ```
pub fn math(latex: impl Into<String>) -> MathBuilder {
    MathBuilder {
        latex: latex.into(),
        color: Color::WHITE,
        font_size: 32.0,
        display: true,
        palette: Vec::new(),
        highlight: DEFAULT_HIGHLIGHT,
        number: 0.0,
        transform: Transform::new(Vec3::ZERO),
    }
}

/// Create a reusable style for math equations: font size, colors, named variable colors and
/// the highlight look. Apply it with [`MathBuilder::style`].
/// ```ignore
/// let style = math_style()
///     .font_size(200.0)
///     .vars([("C", RED), ("S", BLUE), ("q", GREEN)])
///     .highlight(YELLOW, 0.2);
///
/// let drag = math(r"F = \textcolor{C}{C}\textcolor{S}{S}\textcolor{q}{q}")
///     .style(&style)
///     .build();
/// drag.highlight(["q"]) // already yellow with dim 0.2
/// ```
pub fn math_style() -> MathStyle {
    MathStyle::default()
}

/// Reusable styling for math equations, see [`math_style`].
#[derive(Clone, Debug, Default)]
pub struct MathStyle {
    font_size: Option<f32>,
    color: Option<Color>,
    display: Option<bool>,
    palette: Vec<(String, Color)>,
    highlight: Option<(Color, f32)>,
}

#[allow(clippy::return_self_not_must_use)]
impl MathStyle {
    /// Set the font size (em size in pixels)
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
    /// Set the color of the equation
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    /// Use display style or the more compact text style
    pub const fn display(mut self, display: bool) -> Self {
        self.display = Some(display);
        self
    }
    /// Define a named color, like [`MathBuilder::var_color`]
    pub fn var(mut self, name: impl Into<String>, color: Color) -> Self {
        let name = name.into();
        match self.palette.iter_mut().find(|(n, _)| *n == name) {
            Some(entry) => entry.1 = color,
            None => self.palette.push((name, color)),
        }
        self
    }
    /// Define several named colors at once
    pub fn vars<N: Into<String>>(mut self, vars: impl IntoIterator<Item = (N, Color)>) -> Self {
        for (name, color) in vars {
            self = self.var(name, color);
        }
        self
    }
    /// Color and dim used by `highlight(..)` on equations with this style
    pub const fn highlight(mut self, color: Color, dim: f32) -> Self {
        self.highlight = Some((color, dim));
        self
    }
}

/// Builder for a math equation object.
#[must_use]
pub struct MathBuilder {
    latex: String,
    color: Color,
    font_size: f32,
    display: bool,
    palette: Vec<(String, Color)>,
    highlight: (Color, f32),
    number: f32,
    transform: Transform,
}

#[allow(clippy::return_self_not_must_use)]
impl MathBuilder {
    /// Set the color of the equation
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the font size (em size in pixels) of the equation
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Use display style (default) or the more compact text style
    pub const fn display(mut self, display: bool) -> Self {
        self.display = display;
        self
    }
    /// Define a named color for use in the equation: `\textcolor{name}{...}` or `\color{name}`.
    /// Symbols colored this way keep their color; the rest follow the object's color.
    /// ```ignore
    /// math(r"\textcolor{a}{a}x^2 + \textcolor{b}{b}x + c")
    ///     .var_color("a", Color::RED)
    ///     .var_color("b", Color::BLUE)
    /// ```
    pub fn var_color(mut self, name: impl Into<String>, color: Color) -> Self {
        let name = name.into();
        match self.palette.iter_mut().find(|(n, _)| *n == name) {
            Some(entry) => entry.1 = color,
            None => self.palette.push((name, color)),
        }
        self
    }
    /// Color and dim used by [`MathHandle::highlight`] when the animation does not set its own.
    pub const fn highlight_style(mut self, color: Color, dim: f32) -> Self {
        self.highlight = (color, dim);
        self
    }
    /// Apply a reusable [`MathStyle`]. Only what the style sets is changed, and later calls on
    /// this builder win over the style. Style variables that the latex never uses
    /// (`\textcolor{name}`) are skipped, so one style can serve many equations.
    /// ```ignore
    /// let style = math_style().font_size(200.0).var("q", GREEN).highlight(YELLOW, 0.2);
    /// math(r"\textcolor{q}{q} = v^2").style(&style).build()
    /// ```
    pub fn style(mut self, style: &MathStyle) -> Self {
        if let Some(size) = style.font_size {
            self.font_size = size;
        }
        if let Some(color) = style.color {
            self.color = color;
        }
        if let Some(display) = style.display {
            self.display = display;
        }
        if let Some(highlight) = style.highlight {
            self.highlight = highlight;
        }
        for (name, color) in &style.palette {
            if self.latex.contains(&format!("{{{name}}}")) {
                self = self.var_color(name.clone(), *color);
            }
        }
        self
    }
    /// Set the number shown by `{:spec}` placeholders in the latex, e.g. `{:.1}` (default 0).
    /// Animate it with [`AnimObj::number`]. See [`crate::number_format`].
    /// ```ignore
    /// math(r"\alpha = {:.1}^\circ").number(5.0).build()
    /// ```
    pub const fn number(mut self, number: f32) -> Self {
        self.number = number;
        self
    }
    #[must_use]
    pub fn build(self) -> MathHandle {
        let id = self.transform.uuid;
        MathHandle(
            AnimObj {
                id,
                transform: self.transform,
                kind: AnimObjKind::Math {
                    palette: self.palette,
                    latex: self.latex,
                    color: self.color,
                    font_size: self.font_size,
                    display: self.display,
                    number: self.number,
                },
            },
            self.highlight,
        )
    }
}

impl_transform_methods!(MathBuilder);

/// Create a new text template builder.
///
/// A text template stores default text styling that can be reused to create
/// multiple text objects. Each object created from the template can selectively
/// override any property.
///
/// ```ignore
/// let templ = text_template()
///     .font_size(50.0)
///     .font_family("sans-serif")
///     .color(Color::WHITE)
///     .modifier(text_modifier().color(Color::BLACK).thickness(2.0).build())
///     .build();
///
/// let text = templ
///     .text()
///     .value("Hello")
///     .pos(Vec2::new(100.0, 100.0))
///     .font_size(75.0) // overrides template default
///     .build();
/// ```
pub fn text_template() -> TextTemplateBuilder {
    TextTemplateBuilder::default()
}

/// Default text styling that can be reused across multiple text objects.
#[derive(Clone, Debug)]
pub struct TextTemplate {
    font_family: String,
    align: Alignment,
    color: Color,
    font_size: f32,
    modifications: Vec<TextModifier>,
}

impl TextTemplate {
    /// Create a new [`TextBuilder`] pre-filled with this template's defaults.
    /// Any property can still be overridden before calling [`TextBuilder::build`].
    #[must_use]
    pub fn text(&self) -> TextBuilder {
        TextBuilder {
            value: String::new(),
            font_family: self.font_family.clone(),
            align: self.align.clone(),
            color: self.color,
            font_size: self.font_size,
            transform: Transform::new(Vec3::ZERO),
            modifications: self.modifications.clone(),
            spans: Vec::new(),
            number: 0.0,
        }
    }
}

/// Builder for constructing a [`TextTemplate`].
#[must_use]
#[derive(Clone, Debug)]
pub struct TextTemplateBuilder {
    font_family: String,
    align: Alignment,
    color: Color,
    font_size: f32,
    modifications: Vec<TextModifier>,
}

impl Default for TextTemplateBuilder {
    fn default() -> Self {
        Self {
            font_family: "sans-serif".to_string(),
            align: Alignment::Center,
            color: Color::WHITE,
            font_size: 24.0,
            modifications: Vec::new(),
        }
    }
}

impl TextTemplateBuilder {
    /// Set the default font family.
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the default alignment.
    pub const fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// Set the default color.
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the default font size.
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Add a default text modifier.
    pub fn modifier(mut self, modifier: TextModifier) -> Self {
        self.modifications.push(modifier);
        self
    }
    /// Build the template.
    #[must_use]
    pub fn build(self) -> TextTemplate {
        TextTemplate {
            font_family: self.font_family,
            align: self.align,
            color: self.color,
            font_size: self.font_size,
            modifications: self.modifications,
        }
    }
}

/// Create a new text modifier builder for a shadow, outline, or glow effect.
/// ```ignore
/// text()
///     .value("Hello")
///     .modifier(
///         text_modifier()
///             .color(Color::BLACK)
///             .thickness(2.0)
///             .softness(4.0)
///             .pos(Vec2::new(3.0, 3.0))
///             .build()
///     )
///     .build(),
/// ```
pub fn text_modifier() -> TextModifierBuilder {
    TextModifierBuilder {
        color: Color::BLACK,
        thickness: 0.0,
        softness: 0.0,
        pos_offset: Vec3::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    }
}

/// Builder for constructing a [`TextModifier`].
#[must_use]
pub struct TextModifierBuilder {
    color: Color,
    thickness: f32,
    softness: f32,
    pos_offset: Vec3,
    rotation: f32,
    scale: Vec2,
}

#[allow(clippy::return_self_not_must_use)]
impl TextModifierBuilder {
    /// Set the color of the modifier effect
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Set the thickness of the effect — positive expands outward (shadow/outline),
    /// negative contracts inward (inset).
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }
    /// Set the softness/blur amount. Higher values produce softer, more diffuse edges
    /// (useful for soft shadows and glows).
    pub const fn softness(mut self, softness: f32) -> Self {
        self.softness = softness;
        self
    }
    /// Set the 2D position offset relative to the text
    pub const fn pos(mut self, pos: Vec2) -> Self {
        self.pos_offset = pos.extend(self.pos_offset.z);
        self
    }
    /// Set the rotation in degrees
    pub const fn rot(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
    /// Set the scale factor
    pub const fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }
    /// Build the modifier
    pub const fn build(self) -> TextModifier {
        TextModifier {
            color: self.color,
            thickness: self.thickness,
            softness: self.softness,
            pos_offset: self.pos_offset,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

/// Create a new code window object builder.
/// ```ignore
/// code_window()
///     .source("print('hello world')")
///     .syntax(Syntax::Python)
///     .title("demo.py")
///     .build(),
/// ```
pub fn code_window() -> CodeWindowBuilder {
    CodeWindowBuilder::default()
}

#[must_use]
pub struct CodeWindowBuilder {
    source_code: String,
    font_family: String,
    font_size: f32,
    syntax: Syntax,
    theme: Option<Theme>,
    title: String,
    title_font_size: f32,
    width: f32,
    height: f32,
    background_color: Color,
    show_line_numbers: bool,
    line_number_color: Color,
    transform: Transform,
}

impl Default for CodeWindowBuilder {
    fn default() -> Self {
        Self {
            source_code: String::new(),
            font_family: "sans-serif".to_string(),
            font_size: 20.0,
            syntax: Syntax::Rust,
            theme: None,
            title: String::new(),
            title_font_size: 16.0,
            width: 800.0,
            height: 600.0,
            // #11121d, the background of the default theme
            background_color: Color::new(0.067, 0.071, 0.114, 1.0),
            show_line_numbers: false,
            line_number_color: Color::new(0.5, 0.5, 0.5, 0.6),
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

impl CodeWindowBuilder {
    /// Set the source code of the code window
    pub fn source(mut self, code: impl Into<String>) -> Self {
        self.source_code = code.into();
        self
    }
    /// Set the font family of the code window
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the font size of the code window
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Set the syntax highlighting language of the code window
    pub const fn syntax(mut self, syntax: Syntax) -> Self {
        self.syntax = syntax;
        self
    }
    /// Set the theme of the code window
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
    /// Set the title of the code window
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    /// Set the title font size
    pub const fn title_font_size(mut self, size: f32) -> Self {
        self.title_font_size = size;
        self
    }
    /// Set the width of the code window
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Set the height of the code window
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// Set the background color of the code window
    pub const fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }
    /// Toggle line numbers in the code window
    pub const fn line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
    #[must_use]
    pub fn build(self) -> CodeWindowHandle {
        let id = self.transform.uuid;
        let code_id = Uuid::new_v5(&id, b"code");
        let close_btn_id = Uuid::new_v5(&id, b"close");
        let minimize_btn_id = Uuid::new_v5(&id, b"minimize");
        let maximize_btn_id = Uuid::new_v5(&id, b"maximize");
        let title_id = Uuid::new_v5(&id, b"title");
        let bg_id = Uuid::new_v5(&id, b"bg");
        let container_id = Uuid::new_v5(&id, b"container");
        let title_bar_bg_id = Uuid::new_v5(&id, b"title_bg");
        CodeWindowHandle(AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::CodeWindow {
                source_code: self.source_code,
                font_family: self.font_family,
                font_size: self.font_size,
                syntax: self.syntax,
                theme: self.theme,
                title: self.title,
                title_font_size: self.title_font_size,
                width: self.width,
                height: self.height,
                background_color: self.background_color,
                code_id,
                close_btn_id,
                minimize_btn_id,
                maximize_btn_id,
                title_id,
                bg_id,
                container_id,
                title_bar_bg_id,
                show_line_numbers: self.show_line_numbers,
                line_number_color: self.line_number_color,
            },
        })
    }
}

impl_transform_methods!(CodeWindowBuilder);

/// Create a new terminal emulator window builder.
/// ```ignore
/// terminal()
///     .shell("bash")
///     .prompt("$ ")
///     .command("ls -la")
///     .font_family("JetBrains Mono")
///     .build()
/// ```
pub fn terminal() -> TerminalBuilder {
    TerminalBuilder::default()
}

#[must_use]
pub struct TerminalBuilder {
    shell: String,
    prompt: String,
    font_family: String,
    font_size: f32,
    theme: Option<Theme>,
    width: f32,
    height: f32,
    background_color: Color,
    text_color: Color,
    source_dir: Option<String>,
    title: String,
    title_font_size: f32,
    startup_config: Option<String>,
    transform: Transform,
}

impl Default for TerminalBuilder {
    fn default() -> Self {
        Self {
            shell: "bash".to_string(),
            prompt: "$ ".to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            theme: None,
            width: 800.0,
            height: 500.0,
            background_color: Color::new(0.08, 0.08, 0.08, 1.0),
            text_color: Color::new(0.8, 0.8, 0.8, 1.0),
            source_dir: None,
            title: "Terminal".to_string(),
            title_font_size: 25.0,
            startup_config: None,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

impl TerminalBuilder {
    /// Set the shell to use for command execution (e.g. "bash", "fish", "zsh")
    pub fn shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = shell.into();
        self
    }
    /// Set the prompt string (e.g. "$ ", "❯ ", "% ")
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }
    /// Set the font family
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the font size
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Set the theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
    /// Set the width of the terminal window
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Set the height of the terminal window
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// Set the background color
    pub const fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }
    /// Set the default text color
    pub const fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }
    /// Set a source directory whose contents will be copied into the temp working directory
    pub fn source_dir(mut self, dir: impl Into<String>) -> Self {
        self.source_dir = Some(dir.into());
        self
    }
    /// Set the title displayed in the terminal window's title bar
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    /// Set the title bar font size
    pub const fn title_font_size(mut self, size: f32) -> Self {
        self.title_font_size = size;
        self
    }
    /// Set a startup config that will be sourced before each command.
    /// For example: `"starship init fish | source\nstarship preset bracketed-segments -o ~/.config/starship.toml"`
    pub fn startup_config(mut self, config: impl Into<String>) -> Self {
        self.startup_config = Some(config.into());
        self
    }
    #[must_use]
    pub fn build(self) -> TerminalHandle {
        let id = self.transform.uuid;
        let text_buffer_id = Uuid::new_v5(&id, b"text_buffer");
        let bg_id = Uuid::new_v5(&id, b"bg");
        let container_id = Uuid::new_v5(&id, b"container");
        let close_btn_id = Uuid::new_v5(&id, b"close");
        let minimize_btn_id = Uuid::new_v5(&id, b"minimize");
        let maximize_btn_id = Uuid::new_v5(&id, b"maximize");
        let title_id = Uuid::new_v5(&id, b"title");
        let title_bar_bg_id = Uuid::new_v5(&id, b"title_bg");
        TerminalHandle(AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Terminal {
                shell: self.shell,
                prompt: self.prompt,
                font_family: self.font_family,
                font_size: self.font_size,
                theme: self.theme,
                width: self.width,
                height: self.height,
                background_color: self.background_color,
                text_color: self.text_color,
                term_id: id,
                text_buffer_id,
                bg_id,
                container_id,
                close_btn_id,
                minimize_btn_id,
                maximize_btn_id,
                title_id,
                title_bar_bg_id,
                title: self.title,
                title_font_size: self.title_font_size,
                source_dir: self.source_dir,
                startup_config: self.startup_config,
            },
        })
    }
}

impl_transform_methods!(TerminalBuilder);

/// Create a new layout container builder.
/// ```ignore
/// layout()
///     .direction(LayoutDir::Column)
///     .align(Alignment::Center)
///     .gap(10.0)
///     .padding(20.0)
///     .background_color(Color::new(0.1, 0.1, 0.2, 1.0))
///     .corner_radius(8.0)
///     .item(rect1)
///     .item(rect2)
///     .build(),
/// ```
pub fn layout() -> LayoutBuilder {
    LayoutBuilder::default()
}

#[must_use]
pub struct LayoutBuilder {
    children: Vec<AnimObj>,
    direction: LayoutDir,
    align: Alignment,
    justify: Alignment,
    gap: f32,
    padding_top: f32,
    padding_bottom: f32,
    padding_left: f32,
    padding_right: f32,
    min_width: f32,
    min_height: f32,
    background_color: Color,
    corner_radius: f32,
    transform: Transform,
}

impl Default for LayoutBuilder {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            direction: LayoutDir::Column,
            align: Alignment::Center,
            justify: Alignment::Start,
            gap: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            background_color: Color::new(0.15, 0.15, 0.2, 1.0),
            corner_radius: 0.0,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl LayoutBuilder {
    /// Add an item to the layout
    pub fn item(mut self, obj: AnimObj) -> Self {
        self.children.push(obj);
        self
    }
    /// Set the layout direction (column or row)
    pub const fn direction(mut self, dir: LayoutDir) -> Self {
        self.direction = dir;
        self
    }
    /// Set the cross-axis alignment for items (CSS align-items)
    pub const fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// Set the main-axis alignment for items (CSS justify-content)
    pub const fn justify(mut self, a: Alignment) -> Self {
        self.justify = a;
        self
    }
    /// Set the gap between items
    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    /// Set uniform padding on all sides
    pub const fn padding(mut self, pad: f32) -> Self {
        self.padding_top = pad;
        self.padding_bottom = pad;
        self.padding_left = pad;
        self.padding_right = pad;
        self
    }
    /// Set top padding
    pub const fn padding_top(mut self, pad: f32) -> Self {
        self.padding_top = pad;
        self
    }
    /// Set bottom padding
    pub const fn padding_bottom(mut self, pad: f32) -> Self {
        self.padding_bottom = pad;
        self
    }
    /// Set left padding
    pub const fn padding_left(mut self, pad: f32) -> Self {
        self.padding_left = pad;
        self
    }
    /// Set right padding
    pub const fn padding_right(mut self, pad: f32) -> Self {
        self.padding_right = pad;
        self
    }
    /// Set minimum width of the layout background
    pub const fn min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
    /// Set minimum height of the layout background
    pub const fn min_height(mut self, h: f32) -> Self {
        self.min_height = h;
        self
    }
    /// Set the background color
    pub const fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }
    /// Set the corner radius of the background rect
    pub const fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    #[must_use]
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Layout {
                children: self.children,
                direction: self.direction,
                align: self.align,
                justify: self.justify,
                gap: self.gap,
                padding_top: self.padding_top,
                padding_bottom: self.padding_bottom,
                padding_left: self.padding_left,
                padding_right: self.padding_right,
                min_width: self.min_width,
                min_height: self.min_height,
                background_color: self.background_color,
                corner_radius: self.corner_radius,
            },
        }
    }
}

impl_transform_methods!(LayoutBuilder);

/// Create a new invisible padding/spacer object to add spacing between items in a layout.
/// ```ignore
/// layout()
///     .direction(LayoutDir::Column)
///     .gap(10.0)
///     .item(rect1)
///     .item(padding().height(30.0).build())
///     .item(rect2)
///     .build(),
/// ```
pub fn padding() -> PaddingBuilder {
    PaddingBuilder::default()
}

#[must_use]
pub struct PaddingBuilder {
    size: Vec2,
    transform: Transform,
}

impl Default for PaddingBuilder {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl PaddingBuilder {
    /// Set the width of the padding
    pub const fn width(mut self, w: f32) -> Self {
        self.size.x = w;
        self
    }
    /// Set the height of the padding
    pub const fn height(mut self, h: f32) -> Self {
        self.size.y = h;
        self
    }
    /// Set the size of the padding
    pub const fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    #[must_use]
    pub const fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Padding { size: self.size },
        }
    }
}

impl_transform_methods!(PaddingBuilder);

/// Create a new scrollable layout container builder.
/// ```ignore
/// scrol_layout()
///     .direction(LayoutDir::Column)
///     .viewport(400.0, 600.0)
///     .padding(20.0)
///     .gap(10.0)
///     .background_color(Color::new(0.1, 0.1, 0.2, 1.0))
///     .item(text1)
///     .item(text2)
///     .show_scrollbar(true)
///     .mask_children(true)
///     .build(),
/// ```
pub fn scrol_layout() -> ScrollLayoutBuilder {
    ScrollLayoutBuilder::default()
}

#[must_use]
pub struct ScrollLayoutBuilder {
    children: Vec<AnimObj>,
    direction: LayoutDir,
    align: Alignment,
    justify: Alignment,
    gap: f32,
    padding_top: f32,
    padding_bottom: f32,
    padding_left: f32,
    padding_right: f32,
    viewport_width: f32,
    viewport_height: f32,
    show_scrollbar: bool,
    scroll_offset: f32,
    background_color: Color,
    corner_radius: f32,
    mask_children: bool,
    transform: Transform,
}

impl Default for ScrollLayoutBuilder {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            direction: LayoutDir::Column,
            align: Alignment::Center,
            justify: Alignment::Start,
            gap: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
            viewport_width: 300.0,
            viewport_height: 300.0,
            show_scrollbar: false,
            scroll_offset: 0.0,
            background_color: Color::new(0.0, 0.0, 0.0, 0.0),
            corner_radius: 0.0,
            mask_children: true,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl ScrollLayoutBuilder {
    /// Add an item to the scroll layout
    pub fn item(mut self, obj: AnimObj) -> Self {
        self.children.push(obj);
        self
    }
    /// Set the layout direction (column or row)
    pub const fn direction(mut self, dir: LayoutDir) -> Self {
        self.direction = dir;
        self
    }
    /// Set the cross-axis alignment for items (CSS align-items)
    pub const fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// Set the main-axis alignment for items (CSS justify-content)
    pub const fn justify(mut self, a: Alignment) -> Self {
        self.justify = a;
        self
    }
    /// Set the gap between items
    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    /// Set uniform padding on all sides
    pub const fn padding(mut self, pad: f32) -> Self {
        self.padding_top = pad;
        self.padding_bottom = pad;
        self.padding_left = pad;
        self.padding_right = pad;
        self
    }
    /// Set the viewport size (visible area)
    pub const fn viewport(mut self, w: f32, h: f32) -> Self {
        self.viewport_width = w;
        self.viewport_height = h;
        self
    }
    /// Show or hide the scrollbar
    pub const fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }
    /// Enable or disable masking of children outside the viewport.
    /// When enabled (default), child objects are clipped to the viewport bounds.
    pub const fn mask_children(mut self, mask: bool) -> Self {
        self.mask_children = mask;
        self
    }
    /// Set initial scroll offset in pixels
    pub const fn scroll_offset(mut self, offset: f32) -> Self {
        self.scroll_offset = offset;
        self
    }
    /// Set background color of the scroll viewport
    pub const fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }
    /// Set corner radius of the viewport background
    pub const fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    /// Set top padding
    pub const fn padding_top(mut self, pad: f32) -> Self {
        self.padding_top = pad;
        self
    }
    /// Set bottom padding
    pub const fn padding_bottom(mut self, pad: f32) -> Self {
        self.padding_bottom = pad;
        self
    }
    /// Set left padding
    pub const fn padding_left(mut self, pad: f32) -> Self {
        self.padding_left = pad;
        self
    }
    /// Set right padding
    pub const fn padding_right(mut self, pad: f32) -> Self {
        self.padding_right = pad;
        self
    }
    #[must_use]
    pub fn build(self) -> ScrollLayoutHandle {
        let id = self.transform.uuid;
        ScrollLayoutHandle(AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::ScrollLayout {
                children: self.children,
                direction: self.direction,
                align: self.align,
                justify: self.justify,
                gap: self.gap,
                padding_top: self.padding_top,
                padding_bottom: self.padding_bottom,
                padding_left: self.padding_left,
                padding_right: self.padding_right,
                viewport_width: self.viewport_width,
                viewport_height: self.viewport_height,
                show_scrollbar: self.show_scrollbar,
                scroll_offset: self.scroll_offset,
                background_color: self.background_color,
                corner_radius: self.corner_radius,
                mask_children: self.mask_children,
            },
        })
    }
}

impl_transform_methods!(ScrollLayoutBuilder);

#[cfg(test)]
mod vector_tests {
    use crate::{AnimOP, anim_obj::AnimObjKind, prelude::*};
    use glam::vec2;

    #[test]
    fn points_are_stored_around_the_middle_and_defaults_are_filled_in() {
        let v = vector()
            .points([vec2(100., 100.), vec2(300., 200.)])
            .thickness(10.0)
            .dashed(true)
            .build();
        assert_eq!(v.transform.position.truncate(), vec2(200., 150.));
        let AnimObjKind::Vector { points, arrow_head, head_length, head_width, dash, .. } = &v.kind
        else {
            panic!("not a vector");
        };
        assert_eq!(points, &[vec2(-100., -50.), vec2(100., 50.)]);
        assert!(arrow_head);
        assert_eq!((*head_length, *head_width), (50.0, 40.0));
        assert_eq!(*dash, Some((30.0, 20.0)));

        let explicit = vector().points([vec2(0., 0.), vec2(1., 1.)]).dash(8.0, 4.0).build();
        let AnimObjKind::Vector { dash, .. } = &explicit.kind else { panic!() };
        assert_eq!(*dash, Some((8.0, 4.0)));
    }

    #[test]
    fn moving_points_builds_one_op_and_checks_indices() {
        let v = vector().points([vec2(0., 0.), vec2(10., 0.), vec2(20., 0.)]).build();
        let op: AnimOP = v
            .move_point(1, vec2(10., 50.))
            .point(2, vec2(20., 90.))
            .point(1, vec2(10., 60.))
            .over(2.0)
            .into();
        let AnimOP::VectorMovePoints(id, moves, duration, _, _) = op else { panic!() };
        assert_eq!(id, v.transform.uuid);
        // Moving a point twice keeps the last target.
        assert_eq!(moves, [(1, vec2(10., 60.)), (2, vec2(20., 90.))]);
        assert_eq!(duration, 2.0);

        let all = v.points_to([vec2(1., 1.), vec2(2., 2.), vec2(3., 3.)]);
        let AnimOP::VectorMovePoints(_, moves, ..) = all.into() else { panic!() };
        assert_eq!(moves.len(), 3);
    }

    #[test]
    #[should_panic(expected = "there is no point 3")]
    fn moving_a_missing_point_panics() {
        let v = vector().points([vec2(0., 0.), vec2(10., 0.), vec2(20., 0.)]).build();
        let _ = v.move_point(3, vec2(0., 0.));
    }

    #[test]
    #[should_panic(expected = "at least two points")]
    fn needs_two_points() {
        let _ = vector().points([vec2(0., 0.)]).build();
    }
}
