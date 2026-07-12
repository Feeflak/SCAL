#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//! ### Easy animation system centered around code.   
//! // TODO: Preview Window Gif
//!
//! # Example
//! start by creating a new rust app and import 3 crates:
//! ``` toml
//! /// /Cargol.toml
//! [dependencies]
//! # contains functions and types for defining animations
//! # that will be sent to the scal-runtime for rendering/preview.
//! scal-core = "..."
//! # used for communicating with the scal runtime. you only  use ``#[scal_ipc::main]`` from it
//! scal-ipc = "..."
//! # `glam` is a simple and fast linear algebra library for games and graphics.
//! glam = "0.33.2"
//! ```
//! define the animation with:
//! ```
//! /// /src/main.rs
//! use glam::{Vec2, vec2};
//! use scal_core::prelude::*;
//! // Size of the virtual canvas not the output resolution - configured by Config.toml
//! const WINDOW: Vec2 = vec2(1920., 1080.);
//! // This handles all the ipc communication with the scal runtime
//! #[scal_ipc::main]
//! fn main() -> Project {
//!     // https://github.com/tinted-theming/schemes
//!     let theme = Theme::from_base16(Base16::from_hex([
//!         0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
//!         0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
//!     ]));
//!
//!     const CW_WIDTH: f32 = 800.;
//!     const CW_HEIGHT: f32 = 600.;
//!
//!     // Simple objects that handles most needs for animating code.
//!     let cw = code_window()
//!         .source("fn main() {\n    println!(\"Hello, world!\");\n}\n")
//!         .font_family("SF Pro Display")
//!         .font_size(20.)
//!         .syntax(Syntax::Rust)
//!         .line_numbers(true)
//!         .title("fib.rs")
//!         .width(CW_WIDTH)
//!         .height(CW_HEIGHT)
//!         .title_font_size(25.)
//!         .background_color(Color::new(0.15, 0.15, 0.2, 1.))
//!         .pos(WINDOW / 2.)
//!         .build();
//!
//!     let pointer = svg()
//!         .path("./pointer-tool.svg")
//!         .size(Vec2::new(40., 40.))
//!         .color(Color::WHITE)
//!         .stretch(StretchMode::Fit)
//!         .pos(Vec2::new(500., 500.))
//!         .z(1.)
//!         .build();
//!
//!     Project {
//!         scene_settings: SceneSettings {
//!             background_color: Color::new(0.8, 0.8, 0.8, 0.),
//!             camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
//!             default_theme: theme,
//!         },
//!         // This is the actual animation sequence
//!         timeline: timeline![
//!             // all objects need to be instantiated(layout instantiates all children during its instantiation)
//!             cw.instantiate(),
//!             pointer.instantiate(),
//!
//!             wait(0.5.s()),
//!             parallel![
//!                 cw.add_lines()
//!                     .str(
//!                         r"
//! fn fib(n: u32) -> u32 {
//!     match n {
//!         0 => 0,
//!         1 => 1,
//!         _ => fib(n - 1) + fib(n - 2),
//!     }
//! }
//!                 "
//!                     )
//!                     .over(5.s())
//!                     .style(CodeAnimationStyle::TypeWriterInstantResize),
//!             ],
//!             wait(0.5.s()),
//!             pointer
//!                 .transform
//!                 .position()
//!                 .object(cw.close_button())
//!                 .to(vec2(15., 15.))
//!                 .over(0.5.s())
//!                 .ease(Ease::InOutCubic),
//!             cw.close_button().scale().to(Vec2::ONE * 0.8).over(0.3.s()),
//!             cw.close_button().scale().to(Vec2::ONE).over(0.3.s()),
//!             parallel![
//!                 cw.transform
//!                     .scale()
//!                     .to(Vec2::ZERO)
//!                     .over(0.5)
//!                     .ease(Ease::OutCubic),
//!                 cw.transform
//!                     .position()
//!                     .to((WINDOW - vec2(CW_WIDTH, CW_HEIGHT)) / 2.)
//!                     .over(0.5)
//!                     .ease(Ease::OutCubic),
//!             ],
//!             wait(0.5.s()),
//!         ],
//!     }
//! }
//! ```
//! Basic animation output config.
//! ``` toml
//! /// /Config.toml
//! [animation]
//! binary = "cargo run"
//!
//! [rendering]
//! text_resolution_multiplier = 2.0
//! width = 3840
//! height = 2160
//! fps = 60
//!
//! [encoding]
//! output_path = "test.mov"
//! codec_type = "H264Nvenc"
//! ```
//! And now you can just use the scal runtime to render/preview the animation.
//! ``` bash
//! ❯ ls
//! Cargo.toml  Config.toml  pointer-tool.svg  src  test.mov
//! ❯ scal render
//! ...
//! ❯ ffplay ./test.mov
//! ```
//! // TODO: Animation Gif
//!
//! # Features
//!
//! ## High Quality Animations of Code with Ease.
//!
//! ## Hot Reloading Animation Preview
//!
//! ## LSP
//!
//! ## Examples
//!
//! ## Simple Syntax
//!
//! ## Fast Render Times
//!
//!
//! ## Multi-platform
//!
//!
//! # Installing Scala Runtime
//!
//!SCAL uses Cargo to manage all Rust dependencies automatically. Only a few native libraries must be installed manually.
//!
//!## 1. Install Rust
//!
//!If you don't already have Rust installed:
//!
//!```bash
//!curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
//!```
//!
//!Verify the installation:
//!
//!```bash
//!cargo --version
//!rustc --version
//!```
//!
//!---
//!
//!# Linux
//!
//!## Arch Linux
//!
//!Install the required packages:
//!
//!```bash
//!sudo pacman -S \
//!    rustup \
//!    ffmpeg \
//!    pkgconf \
//!    clang \
//!    llvm \
//!    wayland \
//!    libxkbcommon \
//!    libx11 \
//!    libxcursor \
//!    libxi \
//!    libxrandr \
//!    libxcb \
//!    alsa-lib \
//!    libpulse
//!```
//!
//!Initialize Rust (only once):
//!
//!```bash
//!rustup default stable
//!```
//!
//!---
//!
//!## Fedora
//!
//!Install the required packages:
//!
//!```bash
//!sudo dnf install \
//!    rustup \
//!    ffmpeg-devel \
//!    pkgconf-pkg-config \
//!    clang \
//!    llvm-devel \
//!    wayland-devel \
//!    libxkbcommon-devel \
//!    libX11-devel \
//!    libXcursor-devel \
//!    libXi-devel \
//!    libXrandr-devel \
//!    libxcb-devel \
//!    alsa-lib-devel \
//!    pulseaudio-libs-devel
//!```
//!
//!Initialize Rust:
//!
//!```bash
//!rustup default stable
//!```
//!
//!---
//!
//!## Debian / Ubuntu
//!
//!Install the required packages:
//!
//!```bash
//!sudo apt update
//!
//!sudo apt install \
//!    pkg-config \
//!    ffmpeg \
//!    clang \
//!    libclang-dev \
//!    libwayland-dev \
//!    libxkbcommon-dev \
//!    libx11-dev \
//!    libxcursor-dev \
//!    libxi-dev \
//!    libxrandr-dev \
//!    libxcb1-dev \
//!    libasound2-dev \
//!    libpulse-dev
//!```
//!
//!Install Rust:
//!
//!```bash
//!curl https://sh.rustup.rs -sSf | sh
//!```
//!
//!---
//!
//!## NixOS
//!
//!The repository already contains a development shell.
//!
//!Enter it with:
//!
//!```bash
//!nix develop
//!```
//!
//!or
//!
//!```bash
//!nix-shell
//!```
//!
//!All required libraries are provided automatically.
//!
//!---
//!
//!# macOS
//!
//!Install Homebrew if necessary.
//!
//!Then install the required packages:
//!
//!```bash
//!brew install \
//!    rust \
//!    ffmpeg \
//!    llvm \
//!    pkg-config
//!```
//!
//!The remaining libraries required by SCAL are provided by macOS.
//!
//!---
//!
//!# Windows
//!
//!Install:
//!
//!- Rust (via rustup)
//!- Visual Studio 2022 with the **Desktop development with C++** workload
//!- LLVM/Clang
//!- FFmpeg
//!
//!Using Winget:
//!
//!```powershell
//!winget install Rustlang.Rustup
//!
//!winget install LLVM.LLVM
//!
//!winget install Gyan.FFmpeg
//!```
//!
//!After installing Visual Studio Build Tools, restart your terminal.
//!
//!---
//!
//!# Building SCAL
//!
//!Clone the repository:
//!
//!```bash
//!git clone https://github.com/<your-username>/scal.git
//!
//!cd scal
//!```
//!
//!Build:
//!
//!```bash
//!cargo build
//!```
//!
//!Run:
//!
//!```bash
//!cargo run
//!```
//!
//!For optimized release builds:
//!
//!```bash
//!cargo run --release
//!```
//!
//!---
//!
//!# Updating
//!
//!Update the Rust toolchain:
//!
//!```bash
//!rustup update
//!```
//!
//!Update project dependencies:
//!
//!```bash
//!cargo update
//!```
//!
//! # Getting Started
//! sadf.
//! # Setting up a project
//! sadf.
//!

pub mod anim;
pub mod anim_builders;
pub mod anim_obj;
pub mod anim_op;
pub mod camera;
pub mod color;
pub mod ease;
pub mod highlight_specs;
#[allow(missing_docs)]
pub mod object_builders;
pub mod project;
pub mod seconds;
pub mod settings;
pub mod sfx;
pub mod theme;
pub mod transform;

pub use anim_obj::{AnimObj, CodeHandle, CodeWindowHandle, StretchMode, Syntax, TextAlign};
pub use anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, SourceLoc};
pub use camera::Camera;
pub use color::Color;
pub use ease::Ease;
pub use project::{Project, SceneSettings};
pub use scal_ipc_macros::timeline;
pub use seconds::{DurationExt, Time};
pub use settings::{CodecType, EncodingSettings, RenderingSettings};
pub use sfx::{Sfx, SfxBuilder};
pub use theme::{Base16, Theme};
pub use transform::Transform;

/// All the stuff that you need to create animations in Scal
pub mod prelude {
    pub use crate::anim::*;
    pub use crate::anim_builders::*;
    pub use crate::anim_obj::{
        AnimObj, CodeHandle, CodeWindowHandle, StretchMode, Syntax, TextAlign,
    };
    pub use crate::anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp};
    pub use crate::camera::Camera;
    pub use crate::color::Color;
    pub use crate::ease::Ease;
    pub use crate::object_builders::*;
    pub use crate::project::{Project, SceneSettings};
    pub use crate::seconds::DurationExt;
    pub use crate::sfx::{Sfx, sfx};
    pub use crate::theme::{Base16, Theme};
    pub use crate::transform::Transform;
    pub use crate::{parallel, sequence, timeline};
}
