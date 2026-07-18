use glam::{Vec2, Vec3};
use uuid::Uuid;

use crate::anim_obj::{
    AnimObj, AnimObjKind, CodeHandle, CodeWindowHandle, StretchMode, Syntax, TerminalHandle,
    TextAlign,
};
use crate::color::Color;
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
            /// Set the scale of this object's transform
            pub const fn scale(mut self, scale: Vec2) -> Self {
                self.transform.scale = scale;
                self
            }
            /// Set the rotation of this object's transform
            pub const fn rot(mut self, rotation: f32) -> Self {
                self.transform.rotation = rotation;
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
    /// Set the text content
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    /// Set the font family of the text
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    /// Set the alignment of the text
    pub const fn alignment(mut self, align: TextAlign) -> Self {
        self.alignment = align;
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
    #[must_use]
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
            background_color: Color::new(0.176, 0.176, 0.176, 1.0),
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
