#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//! ### Easy animation system focused on code.   
//!
//! # Example
//! start by creating a new rust app and import 3 crates:
//! ``` toml
//! # /Cargol.toml
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
//! ```ignore
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
//! # /Config.toml
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
//! make sure that you have installed required system dependencies:
//!
//! - Ubuntu/Debian:
//!   ``sudo apt install ffmpeg libwayland-dev libxkbcommon-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxcb1-dev libasound2-dev libpulse-dev``
//!
//! - Arch:
//!   ``sudo pacman -S ffmpeg wayland libxkbcommon libx11 libxcursor libxi libxrandr libxcb alsa-lib libpulse``
//!
//! - Fedora:
//!   ``sudo dnf install ffmpeg-devel wayland-devel libxkbcommon-devel libX11-devel libXcursor-devel libXi-devel libXrandr-devel libxcb-devel alsa-lib-devel pulseaudio-libs-devel``
//!
//! - NixOS:
//!   download flake file:
//! ```bash
//! wget https://raw.githubusercontent.com/Feeflak/SCAL/refs/heads/main/flake.nix
//! ```
//! Set-up the environment:
//! ```bash
//! nix develop
//! ```
//!
//! And now you can just use the scal runtime to render/preview the animation.
//! ``` bash
//! ❯ ls
//! Cargo.toml  Config.toml  pointer-tool.svg  src  test.mov
//! ❯ scal render
//! ...
//! ❯ ffplay ./test.mov
//! ```
//! # Features
//!
//! ## High Quality Animations of Code with Ease.
//! First party support for code modification animations, syntax highlighting, custom color schemes, and glyphs.
//! Supports syntax highlighting for most popular languages using Tree-sitter.
//! Use standard [Base16 color schemes](https://github.com/tinted-theming/schemes) to configure
//! syntax highlighting.
//!
//! ## Hot Reloading Animation Preview
//!
//! Supports preview view with a timeline that automatically reloads upon animation code change.
//! Audio waveform display support in timeline.
//! You can click on individual animation operations or sound fx to display what type of animation
//! they are, and where they are located in the animation source code.
//!
//! ## LSP
//! Thanks to the powerful rust LSP you can read documentation from your editor.
//!
//! ## Examples
//!
//! If you want to see features of this project you can look at the example directory.
//!
//! ## Clear Syntax
//! you instantly know what every function does.
//!
//! ```ignore
//! let typing = sfx()
//!     .path("./keeb.wav")
//!     .volume(5.)
//!     .pitch(1.)
//!     .skip_time(0.)
//!     .duration(5.)
//!     .pitch_variation(0.05);
//! ```
//!
//! ## Terminal Animation
//!
//! The terminal object simulates a terminal emulator window for animating CLI interactions.
//! Commands you write in the animation are actually executed on your machine during
//! animation creation, and their real output is captured and displayed.
//!
//! ## Fast Render Times
//! Using FFmpeg supports hardware H264 encoding for most popular GPU brands- nvidia, intel, apple(I
//! only have nvidia gpu-s so tests on other platforms are needed)
//! Uses WGPU to efficiently render scenes.  
//! ## Multi-platform
//! Works on all platforms(Linux, Mac, Windows)
//! (I only have NixOS systems so testing on other platforms is needed)
//!
//! ## Audio System
//! Uses FFmpeg for audio encoding and [cpal](https://github.com/RustAudio/cpal) for preview audio, to deliver stable audio system.
//!
//! # Installing Scala Runtime
//! ## Distro Package
//! Sadly there are no packages currently for this app, you need to compile it from source or
//! install using cargo.
//! ## Nix Flake
//! You can automatically compile from source using flakes like this:
//! ```nix
//! {
//!   inputs.scal-runtime = {
//!     url = "github:Feeflak/scal";
//!     inputs.nixpkgs.follows = "nixpkgs";
//!   };
//!
//!   outputs = { self, nixpkgs, scal-runtime, ... }: {
//!     nixosConfigurations.my-pc = nixpkgs.lib.nixosSystem {
//!       system = "x86_64-linux";
//!
//!       modules = [
//!         ({ pkgs, ... }: {
//!           environment.systemPackages = [
//!             scal-runtime.packages.${pkgs.stdenv.hostPlatform.system}.default
//!           ];
//!         })
//!       ];
//!     };
//!   };
//! }
//! ```
//! ## Compiling From Source
//! SCAL uses Cargo to manage all Rust dependencies automatically. Only a few native libraries must be installed manually.
//!
//! ## 1. Install Rust
//!
//! If you don't already have Rust installed:
//!
//! ```bash
//!     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
//! ```
//!
//! Verify the installation:
//!
//! ```bash
//! cargo --version
//! rustc --version
//! ```
//!
//! ---
//!
//! ## Linux
//!
//! ### Arch Linux
//!
//! Install the required packages:
//!
//! ```bash
//! sudo pacman -S \
//!     ffmpeg \
//!     pkgconf \
//!     clang \
//!     llvm \
//!     wayland \
//!     libxkbcommon \
//!     libx11 \
//!     libxcursor \
//!     libxi \
//!     libxrandr \
//!     libxcb \
//!     alsa-lib \
//!     libpulse
//! ```
//!
//!
//! ---
//!
//! ### Fedora
//!
//! Install the required packages:
//!
//! ```bash
//! sudo dnf install \
//!     ffmpeg-devel \
//!     pkgconf-pkg-config \
//!     clang \
//!     llvm-devel \
//!     wayland-devel \
//!     libxkbcommon-devel \
//!     libX11-devel \
//!     libXcursor-devel \
//!     libXi-devel \
//!     libXrandr-devel \
//!     libxcb-devel \
//!     alsa-lib-devel \
//!     pulseaudio-libs-devel
//! ```
//!
//!
//! ---
//!
//! ## Debian / Ubuntu
//!
//! Install the required packages:
//!
//! ```bash
//! sudo apt update
//!
//! sudo apt install \
//!     pkg-config \
//!     ffmpeg \
//!     clang \
//!     libclang-dev \
//!     libwayland-dev \
//!     libxkbcommon-dev \
//!     libx11-dev \
//!     libxcursor-dev \
//!     libxi-dev \
//!     libxrandr-dev \
//!     libxcb1-dev \
//!     libasound2-dev \
//!     libpulse-dev
//! ```
//!
//!
//! ---
//!
//! ## NixOS
//!
//! The repository already contains a development shell.
//!
//! Enter it with:
//!
//! ```bash
//! nix develop
//! ```
//!
//! or
//!
//! ```bash
//! nix-shell
//! ```
//!
//! All required libraries are provided automatically.
//!
//! ---
//!
//! # macOS
//!
//! Install Homebrew if necessary.
//!
//! Then install the required packages:
//!
//! ```bash
//! brew install \
//!     ffmpeg \
//!     llvm \
//!     pkg-config
//! ```
//!
//! The remaining libraries required by SCAL are provided by macOS.
//!
//! ---
//!
//! # Windows
//!
//! Install:
//!
//! - Rust (via rustup)
//! - Visual Studio 2022 with the **Desktop development with C++** workload
//! - LLVM/Clang
//! - FFmpeg
//!
//! Using Winget:
//!
//! ```powershell
//! winget install Rustlang.Rustup
//!
//! winget install LLVM.LLVM
//!
//! winget install Gyan.FFmpeg
//! ```
//!
//! After installing Visual Studio Build Tools, restart your terminal.
//!
//! ---
//!
//! # Building SCAL
//!
//! ```bash
//! cargo install scal-runtime
//! ```
//!
//! # Setting up a new animation project
//! 1. Init project
//! ```bash
//! cargo init <Name>
//! ````
//! 2. Add dependencies to Cargo.toml
//! ``` toml
//! scal-core = "<version>"
//! scal-ipc = "<version>"
//! glam = "0.33.2"
//! ```
//! 3.  basic animation output config
//! ``` toml
//! # /Config.toml
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
//! 4. setup the ipc macro
//! ```ignore
//! // /scr/main.rs
//! #[scal_ipc::main]
//! fn main() -> Project {
//!     todo!()
//! }
//! ```
//!
//! That's it, you can start writing any animation you want
//! simple example:
//! ```ignore
//! // /scr/main.rs
//! use glam::Vec2;
//! use scal_core::prelude::*;
//!
//! #[scal_ipc::main]
//! fn main() -> Project {
//!     let rect = rectangle()
//!         .size(Vec2::new(600., 400.))
//!         .corner_radius(40.)
//!         .color(Color::new(0., 0.2, 0.4, 1.))
//!         .pos(Vec2::new(400., 540.))
//!         .build();
//!
//!     let circle = circle()
//!         .radius(200.)
//!         .color(Color::new(0.8, 0.2, 0.2, 1.))
//!         .pos(Vec2::new(1200., 500.))
//!         .build();
//!
//!     let hex = polygon()
//!         .radius(180.)
//!         .sides(6)
//!         .color(Color::new(0.2, 0.7, 0.3, 1.))
//!         .pos(Vec2::new(800., 300.))
//!         .build();
//!
//!     let triangle = polygon()
//!         .radius(150.)
//!         .sides(3)
//!         .color(Color::new(0.9, 0.6, 0.1, 1.))
//!         .pos(Vec2::new(1600., 700.))
//!         .build();
//!
//!     Project {
//!         scene_settings: SceneSettings {
//!             background_color: Color::new(0.8, 0.8, 0.8, 1.0),
//!             camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
//!             default_theme: Theme::default(),
//!         },
//!         timeline: timeline![
//!             rect.instantiate(),
//!             circle.instantiate(),
//!             hex.instantiate(),
//!             triangle.instantiate(),
//!             wait(1.s()),
//!             parallel![
//!                 triangle
//!                     .transform
//!                     .position()
//!                     .to(Vec2::new(350., 800.))
//!                     .over(1.s())
//!                     .ease(Ease::OutBack),
//!                 rect.transform
//!                     .position()
//!                     .to(Vec2::new(960., 540.))
//!                     .over(1.s())
//!                     .ease(Ease::InOutBack),
//!             ],
//!         ],
//!     }
//! }
//! ```
//!
//!

/// some basic animation functions/macros
pub mod anim;
/// Builders for animations of some objects
pub mod anim_builders;
/// Generic for all object type. Conversion is done using ``IntoAnimOp`` trait.
pub mod anim_obj;
pub mod anim_op;
/// Camera LOL
pub mod camera;
/// Simple RGBA color struct with some helper functions. each field should be a 0..255 float.
pub mod color;
/// Simple way to make your animations smooth. <https://easings.net/>
pub mod ease;
pub mod highlight_specs;
#[allow(missing_docs)]
pub mod object_builders;
/// Settings
pub mod project;
pub mod seconds;
pub mod settings;
/// Easy way to play sounds
pub mod sfx;
/// Theme for Code and Terminal
pub mod theme;
/// Standard way to control all anim object's position, scale and rotation
pub mod transform;

pub use anim_obj::{
    Alignment, AnimObj, CodeHandle, CodeWindowHandle, LayoutDir, StretchMode, Syntax,
    TerminalHandle, TextAlign, TextModifier,
};
pub use anim_op::{
    AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, SourceLoc, TerminalOutputAction,
};
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
        Alignment, AnimObj, CodeHandle, CodeWindowHandle, LayoutDir, StretchMode, Syntax,
        TerminalHandle, TextAlign, TextModifier,
    };
    pub use crate::anim_op::{
        AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, TerminalOutputAction,
    };
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
