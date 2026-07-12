#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//! ### Easy animation system centered around code.   
//! // TODO: Preview Window Gif
//! // TODO: Move all code from this crate to the runtime
//! // TODO: Add more documentation to the object builders and animation builders
//!
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
//! # Getting Started
//! sadf.
//! # Setting up a project
//! sadf.
//!

// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]

// use crate::anim_object::code_window::code_window;
// use crate::anim_object::text::Align;
// use crate::anim_op::{AnimOperation, CodeHighlight};
// use crate::encoder::{CodecType, EncodingSettings};
// use crate::renderer::RenderingSettings;
// use crate::sfx::{AudioEngine, ScheduledSound};
// use crate::types::{Seconds, Sfx};
// use anyhow::{Context, Result, bail};
// use log::{debug, info};
//
// pub use scal_core::{self, Color, Ease, Seconds as CoreSeconds};
//
// pub mod anim_object;
// pub mod anim_op;
// mod anim_render;
// pub mod animator;
// pub mod audio_player;
// pub mod encoder;
// pub mod nv12;
// pub mod prelude;
// pub mod preview;
// pub mod projection;
// mod readback;
// pub mod renderer;
// pub mod sfx;
// pub mod types;
//
// const BYTES_PER_PIXEL: u32 = 4; //RGBA
// async fn run_loop(
//     tokio_handle: &tokio::runtime::Handle,
//     encoding_settings: EncodingSettings,
//     rendering_settings: RenderingSettings,
//     mut animations: Vec<AnimOperation>,
// ) -> Result<()> {
//     fn op_end_time(
//         op: &AnimOperation,
//         start_time: Seconds,
//         out: &mut Vec<(Sfx, Seconds, Option<scal_core::SourceLoc>)>,
//     ) -> Seconds {
//         match op {
//             AnimOperation::PlaySound(sfx, video_delay, source_loc) => {
//                 let abs_time = start_time + video_delay;
//                 debug!(
//                     "audio: {} at abs_time={}, seek={}",
//                     sfx.path, abs_time, sfx.time_offset
//                 );
//                 out.push((sfx.clone(), abs_time, source_loc.clone()));
//                 start_time
//             }
//             AnimOperation::All(children, _) => {
//                 let mut max_end = start_time;
//                 for child in children {
//                     let end = op_end_time(child, start_time, out);
//                     if end > max_end {
//                         max_end = end;
//                     }
//                 }
//                 max_end
//             }
//             AnimOperation::Sequence(children, _) => {
//                 let mut t = start_time;
//                 for child in children {
//                     t = op_end_time(child, t, out);
//                 }
//                 t
//             }
//             AnimOperation::Wait(dur, _)
//             | AnimOperation::CodeAddLines(_, _, _, dur, _, _, _)
//             | AnimOperation::CodeModifyLine(_, _, _, dur, _, _, _)
//             | AnimOperation::CodeRemoveLines(_, _, dur, _, _, _)
//             | AnimOperation::TransformMovePos(_, _, dur, _, _)
//             | AnimOperation::TransformMoveToObj(_, _, _, dur, _, _)
//             | AnimOperation::TransformRotate(_, _, dur, _, _)
//             | AnimOperation::TransformScale(_, _, dur, _, _) => start_time + dur,
//             AnimOperation::CodeHighlight(_, action, _) => {
//                 start_time + action.duration_and_curve().0
//             }
//             AnimOperation::Instantiate(..) | AnimOperation::Current { .. } => start_time,
//         }
//     }
//
//     let mut sfx_sounds: Vec<(Sfx, Seconds, Option<scal_core::SourceLoc>)> = vec![];
//     let mut time = 0.0;
//     for op in &animations {
//         time = op_end_time(op, time, &mut sfx_sounds);
//     }
//     debug!(
//         "collect_sounds total: {}, total_dur={}",
//         sfx_sounds.len(),
//         time
//     );
//
//     let scheduled: Vec<ScheduledSound> = sfx_sounds
//         .into_iter()
//         .map(|(s, abs_start_time, source_loc_opt)| {
//             let pitch_var = if s.pitch_variation > 0.0 {
//                 use rand::Rng;
//                 let mut rng = rand::thread_rng();
//                 let variation = rng.gen_range(-s.pitch_variation..s.pitch_variation);
//                 s.pitch * (1.0 + variation)
//             } else {
//                 s.pitch
//             };
//             let ss = ScheduledSound {
//                 path: s.path,
//                 volume: s.volume,
//                 pitch: pitch_var,
//                 start_time: abs_start_time,
//                 seek_offset: s.time_offset,
//                 duration: s.duration,
//                 source_loc: source_loc_opt,
//             };
//             debug!(
//                 "ScheduledSound: path={}, start_time={}, seek={}, duration={}, pitch={}",
//                 ss.path, ss.start_time, ss.seek_offset, ss.duration, ss.pitch
//             );
//             ss
//         })
//         .collect();
//     let audio_engine = if scheduled.is_empty() {
//         None
//     } else {
//         Some(AudioEngine::new(scheduled))
//     };
//
//     animations.reverse();
//     info!("Starting rendering loop...");
//     if !(rendering_settings.width * 4).is_multiple_of(256) {
//         bail!("Wgpu needs the bytes_per_row(width * 4) value to be multiple of 256");
//     }
//     let codec_type = encoding_settings.codec_type;
//     let use_nv12 = matches!(codec_type, CodecType::H264 | CodecType::H264Nvenc);
//     let pixel_buffer_size = if use_nv12 {
//         (rendering_settings.width * rendering_settings.height * 3 / 2) as usize
//     } else {
//         (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize
//     };
//     let instance = wgpu::Instance::default();
//
//     let adapter = instance
//         .request_adapter(&wgpu::RequestAdapterOptions::default())
//         .await
//         .context("failed to request wgpu adapter")?;
//
//     info!(
//         "Adapter: {:?} (backend: {:?})",
//         adapter.get_info().name,
//         adapter.get_info().backend
//     );
//
//     let (device, queue) = adapter
//         .request_device(&wgpu::DeviceDescriptor::default())
//         .await
//         .context("failed to request wgpu device")?;
//     readback::init_buffers(rendering_settings.buffer_count, pixel_buffer_size, &device)
//         .context("while initializing buffers")?;
//     let (renderer_send, renderer_rec) =
//         tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
//     for i in 0..rendering_settings.buffer_count as usize {
//         renderer_send.send(i).await.unwrap();
//     }
//     let (encoder_send, encoder_rec) =
//         tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
//     encoder::start_encoding_task(
//         encoding_settings,
//         tokio_handle,
//         rendering_settings,
//         encoder_rec,
//         renderer_send,
//         audio_engine,
//     )
//     .context("while initializing the encoder")?;
//     anim_render::render_animations(
//         queue,
//         animations,
//         readback::ReadbackRing::new(renderer_rec),
//         encoder_send,
//         device,
//         rendering_settings,
//         codec_type,
//     )
//     .await
//     .context("while rendering the animation")?;
//
//     Ok(())
// }
//
// pub async fn render_project(
//     tokio_handle: &tokio::runtime::Handle,
//     core_encoding: scal_core::EncodingSettings,
//     core_rendering: scal_core::RenderingSettings,
//     core_project: scal_core::Project,
// ) -> Result<()> {
//     let encoding = EncodingSettings {
//         output_path: core_encoding.output_path,
//         codec_type: match core_encoding.codec_type {
//             scal_core::CodecType::H264 => CodecType::H264,
//             scal_core::CodecType::H264Nvenc => CodecType::H264Nvenc,
//             scal_core::CodecType::PRORES => CodecType::PRORES,
//         },
//     };
//
//     let rendering = RenderingSettings {
//         camera: core_project.scene_settings.camera,
//         background_color: core_project.scene_settings.background_color,
//         width: core_rendering.width,
//         height: core_rendering.height,
//         fps: core_rendering.fps,
//         buffer_count: core_rendering.buffer_count,
//         text_resolution_multiplier: core_rendering.text_resolution_multiplier,
//     };
//
//     let default_theme = core_project.scene_settings.default_theme;
//     let animations = convert_anim_ops(core_project.timeline, &default_theme)?;
//
//     run_loop(tokio_handle, encoding, rendering, animations).await
// }
//
// pub fn convert_anim_ops(
//     ops: Vec<scal_core::AnimOP>,
//     default_theme: &scal_core::Theme,
// ) -> Result<Vec<AnimOperation>> {
//     let mut result = Vec::with_capacity(ops.len());
//     for op in ops {
//         result.push(convert_anim_op(op, default_theme)?);
//     }
//     Ok(result)
// }
//
// fn convert_anim_op(
//     op: scal_core::AnimOP,
//     default_theme: &scal_core::Theme,
// ) -> Result<AnimOperation> {
//     Ok(match op {
//         scal_core::AnimOP::Wait(dur, loc) => AnimOperation::Wait(dur, loc),
//         scal_core::AnimOP::All(children, loc) => {
//             AnimOperation::All(convert_anim_ops(children, default_theme)?, loc)
//         }
//         scal_core::AnimOP::Sequence(children, loc) => {
//             AnimOperation::Sequence(convert_anim_ops(children, default_theme)?, loc)
//         }
//         scal_core::AnimOP::PlaySound(sfx, delay, loc) => AnimOperation::PlaySound(sfx, delay, loc),
//         scal_core::AnimOP::Instantiate(core_obj, loc) => {
//             if let scal_core::anim_obj::AnimObjKind::CodeWindow { .. } = &core_obj.kind {
//                 let mut op = build_code_window_op(core_obj, default_theme)?;
//                 if let AnimOperation::Instantiate(_, ref mut l) = op {
//                     *l = loc;
//                 }
//                 op
//             } else {
//                 let render_obj = convert_core_anim_obj(core_obj, default_theme)?;
//                 AnimOperation::Instantiate(render_obj, loc)
//             }
//         }
//         scal_core::AnimOP::TransformMovePos(u, v, d, e, loc) => {
//             AnimOperation::TransformMovePos(u, v, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformMoveToObj(u, t, o, d, e, loc) => {
//             AnimOperation::TransformMoveToObj(u, t, o, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformRotate(u, r, d, e, loc) => {
//             AnimOperation::TransformRotate(u, r, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformScale(u, v, d, e, loc) => {
//             AnimOperation::TransformScale(u, v, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::CodeAddLines(u, t, f, d, e, s, loc) => AnimOperation::CodeAddLines(
//             u,
//             t,
//             f,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeModifyLine(u, l, t, d, e, s, loc) => AnimOperation::CodeModifyLine(
//             u,
//             l,
//             t,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeRemoveLines(u, r, d, e, s, loc) => AnimOperation::CodeRemoveLines(
//             u,
//             r,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeHighlight(_, _, _) => {
//             bail!("CodeHighlight conversion not yet implemented")
//         }
//     })
// }
//
// fn convert_anim_op(
//     op: scal_core::AnimOP,
//     default_theme: &scal_core::Theme,
// ) -> Result<AnimOperation> {
//     Ok(match op {
//         scal_core::AnimOP::Wait(dur, loc) => AnimOperation::Wait(dur, loc),
//         scal_core::AnimOP::All(children, loc) => {
//             AnimOperation::All(convert_anim_ops(children, default_theme)?, loc)
//         }
//         scal_core::AnimOP::Sequence(children, loc) => {
//             AnimOperation::Sequence(convert_anim_ops(children, default_theme)?, loc)
//         }
//         scal_core::AnimOP::PlaySound(sfx, delay, loc) => AnimOperation::PlaySound(
//             crate::types::Sfx {
//                 path: sfx.path,
//                 volume: sfx.volume,
//                 pitch: sfx.pitch,
//                 time_offset: sfx.time_offset,
//                 duration: sfx.duration,
//                 pitch_variation: sfx.pitch_variation,
//             },
//             delay,
//             loc,
//         ),
//         scal_core::AnimOP::Instantiate(core_obj, loc) => {
//             if let scal_core::anim_obj::AnimObjKind::CodeWindow { .. } = &core_obj.kind {
//                 let mut op = build_code_window_op(core_obj, default_theme)?;
//                 if let AnimOperation::Instantiate(_, ref mut l) = op {
//                     *l = loc;
//                 }
//                 op
//             } else {
//                 let render_obj = convert_core_anim_obj(core_obj, default_theme)?;
//                 AnimOperation::Instantiate(render_obj, loc)
//             }
//         }
//         scal_core::AnimOP::TransformMovePos(u, v, d, e, loc) => {
//             AnimOperation::TransformMovePos(u, v, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformMoveToObj(u, t, o, d, e, loc) => {
//             AnimOperation::TransformMoveToObj(u, t, o, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformRotate(u, r, d, e, loc) => {
//             AnimOperation::TransformRotate(u, r, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::TransformScale(u, v, d, e, loc) => {
//             AnimOperation::TransformScale(u, v, d, anim_op::convert_curve(e), loc)
//         }
//         scal_core::AnimOP::CodeAddLines(u, t, f, d, e, s, loc) => AnimOperation::CodeAddLines(
//             u,
//             t,
//             f,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeModifyLine(u, l, t, d, e, s, loc) => AnimOperation::CodeModifyLine(
//             u,
//             l,
//             t,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeRemoveLines(u, r, d, e, s, loc) => AnimOperation::CodeRemoveLines(
//             u,
//             r,
//             d,
//             anim_op::convert_curve(e),
//             convert_style(&s),
//             loc,
//         ),
//         scal_core::AnimOP::CodeHighlight(_, _, _) => {
//             bail!("CodeHighlight conversion not yet implemented")
//         }
//     })
// }
//
// const fn convert_style(
//     s: &scal_core::anim_op::CodeAnimationStyle,
// ) -> crate::anim_object::text::code::CodeAnimationStyle {
//     match *s {
//         scal_core::anim_op::CodeAnimationStyle::TypeWriter => {
//             crate::anim_object::text::code::CodeAnimationStyle::TypeWriter
//         }
//         scal_core::anim_op::CodeAnimationStyle::TypeWriterInstantResize => {
//             crate::anim_object::text::code::CodeAnimationStyle::TypeWriterInstantResize
//         }
//         scal_core::anim_op::CodeAnimationStyle::Fold => {
//             crate::anim_object::text::code::CodeAnimationStyle::Fold
//         }
//     }
// }
//
// const fn make_transform(obj: &scal_core::AnimObj) -> crate::anim_object::Transform {
//     crate::anim_object::Transform {
//         scale: obj.transform.scale,
//         uuid: obj.id,
//         parent: obj.transform.parent,
//         position: obj.transform.position,
//         rotation: obj.transform.rotation,
//         layout_container: None,
//         world_uniform: None,
//     }
// }
//
// fn c(color: scal_core::Color) -> crate::types::Color {
//     crate::types::Color::new(color.r, color.g, color.b, color.a)
// }
//
// fn convert_base16(b: &scal_core::Base16) -> crate::anim_object::text::code::theme::Base16 {
//     let mut colors = [crate::types::Color::BLACK; 16];
//     for (i, &col) in b.colors.iter().enumerate() {
//         colors[i] = c(col);
//     }
//     crate::anim_object::text::code::theme::Base16 { colors }
// }
//
// fn build_code_window_op(
//     obj: scal_core::AnimObj,
//     default_theme: &scal_core::Theme,
// ) -> Result<AnimOperation> {
//     use scal_core::anim_obj::{AnimObjKind, Syntax};
//     if let AnimObjKind::CodeWindow {
//         source_code,
//         font_family,
//         font_size,
//         syntax,
//         theme,
//         title,
//         title_font_size,
//         width,
//         height,
//         background_color,
//         code_id,
//         close_btn_id,
//         minimize_btn_id,
//         maximize_btn_id,
//         title_id,
//         container_id,
//         title_bar_bg_id,
//         show_line_numbers,
//         line_number_color,
//         ..
//     } = obj.kind
//     {
//         let syn = match syntax {
//             Syntax::Rust => crate::anim_object::text::code::Syntax::Rust,
//             Syntax::Nix => crate::anim_object::text::code::Syntax::Nix,
//             Syntax::Python => crate::anim_object::text::code::Syntax::Python,
//             Syntax::JS => crate::anim_object::text::code::Syntax::JS,
//             Syntax::Zig => crate::anim_object::text::code::Syntax::Zig,
//         };
//         let t = theme.as_ref().unwrap_or(default_theme);
//         let render_base16 = convert_base16(&t.base);
//         let th = crate::anim_object::text::code::theme::Theme::from_base16(render_base16);
//         let cw = code_window(
//             obj.transform.position,
//             source_code,
//             th,
//             font_family,
//             Align::Left,
//             font_size,
//             syn,
//             title,
//             width,
//             height,
//             title_font_size,
//             c(background_color),
//             code_id,
//             close_btn_id,
//             minimize_btn_id,
//             maximize_btn_id,
//             title_id,
//             obj.id,
//             container_id,
//             title_bar_bg_id,
//             show_line_numbers,
//             c(line_number_color),
//         );
//         Ok(cw.instantiate())
//     } else {
//         bail!("build_code_window_op called on non-CodeWindow kind")
//     }
// }
//
// fn convert_core_anim_obj(
//     obj: scal_core::AnimObj,
//     default_theme: &scal_core::Theme,
// ) -> Result<crate::anim_object::object_trait::AnimObj> {
//     use crate::anim_object::object_trait::AnimObj as RenderObj;
//     let transform = make_transform(&obj);
//     match obj.kind {
//         scal_core::anim_obj::AnimObjKind::Rectangle {
//             size,
//             corner_radius,
//             color,
//         } => Ok(RenderObj(Box::new(
//             crate::anim_object::primitive_shapes::Rectangle {
//                 size,
//                 corner_radius,
//                 color: c(color),
//                 transform,
//             },
//         ))),
//         scal_core::anim_obj::AnimObjKind::Circle { radius, color } => Ok(RenderObj(Box::new(
//             crate::anim_object::primitive_shapes::Circle {
//                 radius,
//                 color: c(color),
//                 transform,
//             },
//         ))),
//         scal_core::anim_obj::AnimObjKind::Polygon {
//             radius,
//             sides,
//             color,
//         } => Ok(RenderObj(Box::new(
//             crate::anim_object::primitive_shapes::Polygon {
//                 radius,
//                 sides,
//                 color: c(color),
//                 transform,
//             },
//         ))),
//         scal_core::anim_obj::AnimObjKind::Text {
//             value,
//             font_family,
//             alignment,
//             color,
//             font_size,
//         } => {
//             let align = match alignment {
//                 scal_core::anim_obj::TextAlign::Center => crate::anim_object::text::Align::Center,
//                 scal_core::anim_obj::TextAlign::Left => crate::anim_object::text::Align::Left,
//                 scal_core::anim_obj::TextAlign::Right => crate::anim_object::text::Align::Right,
//             };
//             Ok(RenderObj(Box::new(crate::anim_object::text::Text {
//                 id: obj.id,
//                 value,
//                 font_family,
//                 alignment: align,
//                 color: c(color),
//                 font_size,
//                 transform,
//                 cached_size: None,
//             })))
//         }
//         scal_core::anim_obj::AnimObjKind::Svg {
//             path,
//             size,
//             tint,
//             fill,
//             stroke,
//             stroke_width,
//             stretch,
//         } => {
//             let st = match stretch {
//                 scal_core::anim_obj::StretchMode::Fit => {
//                     crate::anim_object::image::StretchMode::Fit
//                 }
//                 scal_core::anim_obj::StretchMode::Fill => {
//                     crate::anim_object::image::StretchMode::Fill
//                 }
//             };
//             Ok(RenderObj(Box::new(crate::anim_object::svg::Svg {
//                 path,
//                 size,
//                 tint: c(tint),
//                 fill: fill.map(c),
//                 stroke: stroke.map(c),
//                 stroke_width,
//                 stretch: st,
//                 transform,
//             })))
//         }
//         scal_core::anim_obj::AnimObjKind::Image {
//             path,
//             size,
//             color,
//             stretch,
//         } => {
//             let st = match stretch {
//                 scal_core::anim_obj::StretchMode::Fit => {
//                     crate::anim_object::image::StretchMode::Fit
//                 }
//                 scal_core::anim_obj::StretchMode::Fill => {
//                     crate::anim_object::image::StretchMode::Fill
//                 }
//             };
//             Ok(RenderObj(Box::new(crate::anim_object::image::Image {
//                 path,
//                 size,
//                 color: c(color),
//                 stretch: st,
//                 transform,
//             })))
//         }
//         scal_core::anim_obj::AnimObjKind::Code {
//             source_code,
//             font_family,
//             font_size,
//             syntax,
//             theme,
//             padding,
//             show_line_numbers,
//             line_number_color,
//         } => {
//             let syn = match syntax {
//                 scal_core::anim_obj::Syntax::Rust => crate::anim_object::text::code::Syntax::Rust,
//                 scal_core::anim_obj::Syntax::Nix => crate::anim_object::text::code::Syntax::Nix,
//                 scal_core::anim_obj::Syntax::Python => {
//                     crate::anim_object::text::code::Syntax::Python
//                 }
//                 scal_core::anim_obj::Syntax::JS => crate::anim_object::text::code::Syntax::JS,
//                 scal_core::anim_obj::Syntax::Zig => crate::anim_object::text::code::Syntax::Zig,
//             };
//             let t = theme.as_ref().unwrap_or(default_theme);
//             let render_base16 = convert_base16(&t.base);
//             let th = crate::anim_object::text::code::theme::Theme::from_base16(render_base16);
//             Ok(RenderObj(Box::new(crate::anim_object::text::code::Code {
//                 id: obj.id,
//                 source_code,
//                 theme: th,
//                 font_family,
//                 font_size,
//                 syntax: syn,
//                 padding,
//                 show_line_numbers,
//                 line_number_color: c(line_number_color),
//                 transform,
//                 alignment: crate::anim_object::text::Align::Left,
//                 lines: vec![],
//                 dirty: true,
//                 anim_reveal: 1.0,
//                 anim_spacing: 0.0,
//                 anim_line_start: 0,
//                 anim_line_end: 0,
//                 anim_style: crate::anim_object::text::code::CodeAnimationStyle::TypeWriter,
//                 anim_spacing_accum: 0.0,
//                 cached_size: None,
//                 highlights: vec![],
//             })))
//         }
//         scal_core::anim_obj::AnimObjKind::CodeWindow { .. } => {
//             bail!("CodeWindow should be handled by build_code_window_op")
//         }
//         scal_core::anim_obj::AnimObjKind::Group { .. } => {
//             bail!("Group object conversion not yet implemented")
//         }
//     }
// }
pub mod anim_obj;
pub mod anim_op;
pub mod builders;
pub mod camera;
pub mod color;
pub mod ease;
pub mod highlight_specs;
pub mod project;
pub mod seconds;
pub mod settings;
pub mod sfx;
pub mod theme;
pub mod transform;

pub use anim_obj::{
    AnimObj, CodeHandle, CodeWindowHandle, StretchMode, SubObjectHandle, Syntax, TextAlign,
};
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

pub mod prelude {
    pub use crate::anim_obj::{
        AnimObj, CodeHandle, CodeWindowHandle, StretchMode, SubObjectHandle, Syntax, TextAlign,
    };
    pub use crate::anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, wait};
    pub use crate::builders::*;
    pub use crate::camera::Camera;
    pub use crate::color::Color;
    pub use crate::ease::Ease;
    pub use crate::project::{Project, SceneSettings};
    pub use crate::seconds::DurationExt;
    pub use crate::sfx::{Sfx, sfx};
    pub use crate::theme::{Base16, Theme};
    pub use crate::transform::Transform;
    pub use crate::{parallel, sequence, timeline};
}
