use glam::{Vec2, Vec3};

use crate::anim_obj::{AnimObj, AnimObjKind, StretchMode, Syntax, TextAlign};
use crate::color::Color;
use crate::theme::Theme;
use crate::transform::Transform;

macro_rules! impl_transform_methods {
    ($builder:ty) => {
        impl $builder {
            pub fn pos(mut self, position: Vec2) -> Self {
                self.transform.position = position.extend(self.transform.position.z);
                self
            }
            pub fn z(mut self, z: f32) -> Self {
                self.transform.position.z = z;
                self
            }
            pub fn scale(mut self, scale: Vec2) -> Self {
                self.transform.scale = scale;
                self
            }
            pub fn rot(mut self, rotation: f32) -> Self {
                self.transform.rotation = rotation;
                self
            }
        }
    };
}

pub fn svg() -> SvgBuilder {
    SvgBuilder::default()
}

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

impl SvgBuilder {
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }
    pub fn stretch(mut self, stretch: StretchMode) -> Self {
        self.stretch = stretch;
        self
    }
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

pub fn rectangle() -> RectangleBuilder {
    RectangleBuilder::default()
}

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

impl RectangleBuilder {
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn build(self) -> AnimObj {
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

pub fn circle() -> CircleBuilder {
    CircleBuilder::default()
}

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

impl CircleBuilder {
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn build(self) -> AnimObj {
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

pub fn code() -> CodeBuilder {
    CodeBuilder::default()
}

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

impl CodeBuilder {
    pub fn source(mut self, code: impl Into<String>) -> Self {
        self.source_code = code.into();
        self
    }
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    pub fn syntax(mut self, syntax: Syntax) -> Self {
        self.syntax = syntax;
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
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
        }
    }
}

impl_transform_methods!(CodeBuilder);

pub fn polygon() -> PolygonBuilder {
    PolygonBuilder::default()
}

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
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub fn sides(mut self, sides: u32) -> Self {
        self.sides = sides;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn build(self) -> AnimObj {
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

pub fn image() -> ImageBuilder {
    ImageBuilder::default()
}

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
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn stretch(mut self, stretch: StretchMode) -> Self {
        self.stretch = stretch;
        self
    }
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

pub fn text() -> TextBuilder {
    TextBuilder::default()
}

pub struct TextBuilder {
    value: String,
    font_family: String,
    alignment: TextAlign,
    color: Color,
    font_size: f32,
    transform: Transform,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self {
            value: String::new(),
            font_family: "sans-serif".to_string(),
            alignment: TextAlign::Center,
            color: Color::WHITE,
            font_size: 24.0,
            transform: Transform::new(Vec3::ZERO),
        }
    }
}

impl TextBuilder {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    pub fn alignment(mut self, align: TextAlign) -> Self {
        self.alignment = align;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    pub fn build(self) -> AnimObj {
        let id = self.transform.uuid;
        AnimObj {
            id,
            transform: self.transform,
            kind: AnimObjKind::Text {
                value: self.value,
                font_family: self.font_family,
                alignment: self.alignment,
                color: self.color,
                font_size: self.font_size,
            },
        }
    }
}

impl_transform_methods!(TextBuilder);