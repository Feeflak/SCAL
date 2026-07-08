use glam::Vec2;
use uuid::Uuid;

use crate::anim_obj::{AnimObj, AnimObjKind, StretchMode, Syntax, TextAlign};
use crate::color::Color;
use crate::transform::Transform;

pub fn svg() -> SvgBuilder {
    SvgBuilder {
        path: String::new(),
        size: Vec2::splat(40.0),
        tint: Color::WHITE,
        fill: None,
        stroke: None,
        stroke_width: None,
        stretch: StretchMode::Fit,
    }
}

pub struct SvgBuilder {
    path: String,
    size: Vec2,
    tint: Color,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: Option<f32>,
    stretch: StretchMode,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
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

pub fn rectangle() -> RectangleBuilder {
    RectangleBuilder {
        size: Vec2::splat(100.0),
        corner_radius: 0.0,
        color: Color::WHITE,
    }
}

pub struct RectangleBuilder {
    size: Vec2,
    corner_radius: f32,
    color: Color,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
            kind: AnimObjKind::Rectangle {
                size: self.size,
                corner_radius: self.corner_radius,
                color: self.color,
            },
        }
    }
}

pub fn circle() -> CircleBuilder {
    CircleBuilder {
        radius: 50.0,
        color: Color::WHITE,
    }
}

pub struct CircleBuilder {
    radius: f32,
    color: Color,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
            kind: AnimObjKind::Circle {
                radius: self.radius,
                color: self.color,
            },
        }
    }
}

pub fn code() -> CodeBuilder {
    CodeBuilder {
        source_code: String::new(),
        font_family: "sans-serif".to_string(),
        font_size: 20.0,
        syntax: Syntax::Rust,
        theme: vec![],
        padding: 20.0,
        show_line_numbers: false,
        line_number_color: Color::new(0.5, 0.5, 0.5, 0.6),
    }
}

pub struct CodeBuilder {
    source_code: String,
    font_family: String,
    font_size: f32,
    syntax: Syntax,
    theme: Vec<u32>,
    padding: f32,
    show_line_numbers: bool,
    line_number_color: Color,
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
    pub fn theme(mut self, base16_colors: Vec<u32>) -> Self {
        self.theme = base16_colors;
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
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

pub fn polygon() -> PolygonBuilder {
    PolygonBuilder {
        radius: 50.0,
        sides: 6,
        color: Color::WHITE,
    }
}

pub struct PolygonBuilder {
    radius: f32,
    sides: u32,
    color: Color,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
            kind: AnimObjKind::Polygon {
                radius: self.radius,
                sides: self.sides,
                color: self.color,
            },
        }
    }
}

pub fn image() -> ImageBuilder {
    ImageBuilder {
        path: String::new(),
        size: Vec2::splat(100.0),
        color: Color::WHITE,
        stretch: StretchMode::Fit,
    }
}

pub struct ImageBuilder {
    path: String,
    size: Vec2,
    color: Color,
    stretch: StretchMode,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
            kind: AnimObjKind::Image {
                path: self.path,
                size: self.size,
                color: self.color,
                stretch: self.stretch,
            },
        }
    }
}

pub fn text() -> TextBuilder {
    TextBuilder {
        value: String::new(),
        font_family: "sans-serif".to_string(),
        alignment: TextAlign::Center,
        color: Color::WHITE,
        font_size: 24.0,
    }
}

pub struct TextBuilder {
    value: String,
    font_family: String,
    alignment: TextAlign,
    color: Color,
    font_size: f32,
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
    pub fn create(self, position: Vec2) -> AnimObj {
        AnimObj {
            id: Uuid::new_v4(),
            transform: Transform::new(position.extend(0.0)),
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
