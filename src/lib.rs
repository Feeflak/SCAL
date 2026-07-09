#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use crate::anim_object::code_window::code_window;
use crate::anim_object::text::Align;
use crate::anim_op::AnimOP;
use crate::encoder::{CodecType, EncodingSettings};
use crate::renderer::RenderingSettings;
use crate::sfx::{AudioEngine, ScheduledSound};
use crate::types::{Seconds, Sfx};
use anyhow::{Context, Result, bail};
use log::{debug, info};

pub use scal_core::{self, Color, Ease, Seconds as CoreSeconds};

pub mod anim_object;
pub mod anim_op;
mod anim_render;
pub mod animator;
pub mod encoder;
pub mod nv12;
pub mod prelude;
pub mod projection;
mod readback;
pub mod renderer;
pub mod sfx;
pub mod types;

const BYTES_PER_PIXEL: u32 = 4; //RGBA
async fn run_loop(
    tokio_handle: &tokio::runtime::Handle,
    encoding_settings: EncodingSettings,
    rendering_settings: RenderingSettings,
    mut animations: Vec<AnimOP>,
) -> Result<()> {
    fn op_end_time(op: &AnimOP, start_time: Seconds, out: &mut Vec<(Sfx, Seconds)>) -> Seconds {
        match op {
            AnimOP::PlaySound(sfx, video_delay) => {
                let abs_time = start_time + video_delay;
                debug!(
                    "audio: {} at abs_time={}, seek={}",
                    sfx.path, abs_time, sfx.time_offset
                );
                out.push((sfx.clone(), abs_time));
                start_time
            }
            AnimOP::All(children) => {
                let mut max_end = start_time;
                for child in children {
                    let end = op_end_time(child, start_time, out);
                    if end > max_end {
                        max_end = end;
                    }
                }
                max_end
            }
            AnimOP::Sequence(children) => {
                let mut t = start_time;
                for child in children {
                    t = op_end_time(child, t, out);
                }
                t
            }
            AnimOP::Wait(dur)
            | AnimOP::CodeAddLines(_, _, _, dur, _, _)
            | AnimOP::CodeModifyLine(_, _, _, dur, _, _)
            | AnimOP::CodeRemoveLines(_, _, dur, _, _)
            | AnimOP::TransformMovePos(_, _, dur, _)
            | AnimOP::TransformMoveToObj(_, _, _, dur, _)
            | AnimOP::TransformRotate(_, _, dur, _)
            | AnimOP::TransformScale(_, _, dur, _) => start_time + dur,
            AnimOP::CodeHighlight(_, action) => start_time + action.duration_and_curve().0,
            AnimOP::Instantiate(_) | AnimOP::Current { .. } => start_time,
        }
    }

    let mut sfx_sounds: Vec<(Sfx, Seconds)> = vec![];
    let mut time = 0.0;
    for op in &animations {
        time = op_end_time(op, time, &mut sfx_sounds);
    }
    debug!(
        "collect_sounds total: {}, total_dur={}",
        sfx_sounds.len(),
        time
    );

    let scheduled: Vec<ScheduledSound> = sfx_sounds
        .into_iter()
        .map(|(s, abs_start_time)| {
            let pitch_var = if s.pitch_variation > 0.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let variation = rng.gen_range(-s.pitch_variation..s.pitch_variation);
                s.pitch * (1.0 + variation)
            } else {
                s.pitch
            };
            let ss = ScheduledSound {
                path: s.path,
                volume: s.volume,
                pitch: pitch_var,
                start_time: abs_start_time,
                seek_offset: s.time_offset,
                duration: s.duration,
            };
            debug!(
                "ScheduledSound: path={}, start_time={}, seek={}, duration={}, pitch={}",
                ss.path, ss.start_time, ss.seek_offset, ss.duration, ss.pitch
            );
            ss
        })
        .collect();
    let audio_engine = if scheduled.is_empty() {
        None
    } else {
        Some(AudioEngine::new(scheduled))
    };

    animations.reverse();
    info!("Starting rendering loop...");
    if !(rendering_settings.width * 4).is_multiple_of(256) {
        bail!("Wgpu needs the bytes_per_row(width * 4) value to be multiple of 256");
    }
    let codec_type = encoding_settings.codec_type;
    let use_nv12 = matches!(codec_type, CodecType::H264 | CodecType::H264Nvenc);
    let pixel_buffer_size = if use_nv12 {
        (rendering_settings.width * rendering_settings.height * 3 / 2) as usize
    } else {
        (rendering_settings.width * rendering_settings.height * BYTES_PER_PIXEL) as usize
    };
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .context("failed to request wgpu adapter")?;

    info!(
        "Adapter: {:?} (backend: {:?})",
        adapter.get_info().name,
        adapter.get_info().backend
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .context("failed to request wgpu device")?;
    readback::init_buffers(rendering_settings.buffer_count, pixel_buffer_size, &device)
        .context("while initializing buffers")?;
    let (renderer_send, renderer_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    for i in 0..rendering_settings.buffer_count as usize {
        renderer_send.send(i).await.unwrap();
    }
    let (encoder_send, encoder_rec) =
        tokio::sync::mpsc::channel(rendering_settings.buffer_count as usize);
    encoder::start_encoding_task(
        encoding_settings,
        tokio_handle,
        rendering_settings,
        encoder_rec,
        renderer_send,
        audio_engine,
    )
    .context("while initializing the encoder")?;
    anim_render::render_animations(
        queue,
        animations,
        readback::ReadbackRing::new(renderer_rec),
        encoder_send,
        device,
        rendering_settings,
        codec_type,
    )
    .await
    .context("while rendering the animation")?;

    Ok(())
}

pub async fn render_project(
    tokio_handle: &tokio::runtime::Handle,
    core_encoding: scal_core::EncodingSettings,
    core_rendering: scal_core::RenderingSettings,
    core_project: scal_core::Project,
) -> Result<()> {
    let encoding = EncodingSettings {
        output_path: core_encoding.output_path,
        codec_type: match core_encoding.codec_type {
            scal_core::CodecType::H264 => CodecType::H264,
            scal_core::CodecType::H264Nvenc => CodecType::H264Nvenc,
            scal_core::CodecType::PRORES => CodecType::PRORES,
        },
    };

    let camera = crate::projection::Camera::new(
        core_project.scene_settings.camera.virtual_size,
        core_project.scene_settings.camera.position,
        core_project.scene_settings.camera.zoom,
    );

    let rendering = RenderingSettings {
        camera,
        background_color: crate::types::Color::new(
            core_project.scene_settings.background_color.r,
            core_project.scene_settings.background_color.g,
            core_project.scene_settings.background_color.b,
            core_project.scene_settings.background_color.a,
        ),
        width: core_rendering.width,
        height: core_rendering.height,
        fps: core_rendering.fps,
        buffer_count: core_rendering.buffer_count,
        text_resolution_multiplier: core_rendering.text_resolution_multiplier,
    };

    let default_theme = core_project.scene_settings.default_theme;
    let animations = convert_anim_ops(core_project.timeline, &default_theme)?;

    run_loop(tokio_handle, encoding, rendering, animations).await
}

fn convert_anim_ops(
    ops: Vec<scal_core::AnimOP>,
    default_theme: &scal_core::Theme,
) -> Result<Vec<AnimOP>> {
    let mut result = Vec::with_capacity(ops.len());
    for op in ops {
        result.push(convert_anim_op(op, default_theme)?);
    }
    Ok(result)
}

fn convert_anim_op(op: scal_core::AnimOP, default_theme: &scal_core::Theme) -> Result<AnimOP> {
    Ok(match op {
        scal_core::AnimOP::Wait(dur, _loc) => AnimOP::Wait(dur),
        scal_core::AnimOP::All(children, _loc) => {
            AnimOP::All(convert_anim_ops(children, default_theme)?)
        }
        scal_core::AnimOP::Sequence(children, _loc) => {
            AnimOP::Sequence(convert_anim_ops(children, default_theme)?)
        }
        scal_core::AnimOP::PlaySound(sfx, delay, _loc) => AnimOP::PlaySound(
            crate::types::Sfx {
                path: sfx.path,
                volume: sfx.volume,
                pitch: sfx.pitch,
                time_offset: sfx.time_offset,
                duration: sfx.duration,
                pitch_variation: sfx.pitch_variation,
            },
            delay,
        ),
        scal_core::AnimOP::Instantiate(core_obj, _loc) => {
            if let scal_core::anim_obj::AnimObjKind::CodeWindow { .. } = &core_obj.kind {
                build_code_window_op(core_obj, default_theme)?
            } else {
                let render_obj = convert_core_anim_obj(core_obj, default_theme)?;
                AnimOP::Instantiate(render_obj)
            }
        }
        scal_core::AnimOP::TransformMovePos(u, v, d, e, _loc) => {
            AnimOP::TransformMovePos(u, v, d, anim_op::convert_curve(e))
        }
        scal_core::AnimOP::TransformMoveToObj(u, t, o, d, e, _loc) => {
            AnimOP::TransformMoveToObj(u, t, o, d, anim_op::convert_curve(e))
        }
        scal_core::AnimOP::TransformRotate(u, r, d, e, _loc) => {
            AnimOP::TransformRotate(u, r, d, anim_op::convert_curve(e))
        }
        scal_core::AnimOP::TransformScale(u, v, d, e, _loc) => {
            AnimOP::TransformScale(u, v, d, anim_op::convert_curve(e))
        }
        scal_core::AnimOP::CodeAddLines(u, t, f, d, e, s, _loc) => {
            AnimOP::CodeAddLines(u, t, f, d, anim_op::convert_curve(e), convert_style(&s))
        }
        scal_core::AnimOP::CodeModifyLine(u, l, t, d, e, s, _loc) => {
            AnimOP::CodeModifyLine(u, l, t, d, anim_op::convert_curve(e), convert_style(&s))
        }
        scal_core::AnimOP::CodeRemoveLines(u, r, d, e, s, _loc) => {
            AnimOP::CodeRemoveLines(u, r, d, anim_op::convert_curve(e), convert_style(&s))
        }
        scal_core::AnimOP::CodeHighlight(_, _, _) => {
            bail!("CodeHighlight conversion not yet implemented")
        }
    })
}

const fn convert_style(
    s: &scal_core::anim_op::CodeAnimationStyle,
) -> crate::anim_object::text::code::CodeAnimationStyle {
    match *s {
        scal_core::anim_op::CodeAnimationStyle::TypeWriter => {
            crate::anim_object::text::code::CodeAnimationStyle::TypeWriter
        }
        scal_core::anim_op::CodeAnimationStyle::TypeWriterInstantResize => {
            crate::anim_object::text::code::CodeAnimationStyle::TypeWriterInstantResize
        }
        scal_core::anim_op::CodeAnimationStyle::Fold => {
            crate::anim_object::text::code::CodeAnimationStyle::Fold
        }
    }
}

const fn make_transform(obj: &scal_core::AnimObj) -> crate::anim_object::Transform {
    crate::anim_object::Transform {
        scale: obj.transform.scale,
        uuid: obj.id,
        parent: obj.transform.parent,
        position: obj.transform.position,
        rotation: obj.transform.rotation,
        layout_container: None,
        world_uniform: None,
    }
}

fn c(color: scal_core::Color) -> crate::types::Color {
    crate::types::Color::new(color.r, color.g, color.b, color.a)
}

fn convert_base16(b: &scal_core::Base16) -> crate::anim_object::text::code::theme::Base16 {
    let mut colors = [crate::types::Color::BLACK; 16];
    for (i, &col) in b.colors.iter().enumerate() {
        colors[i] = c(col);
    }
    crate::anim_object::text::code::theme::Base16 { colors }
}

fn build_code_window_op(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<AnimOP> {
    use scal_core::anim_obj::{AnimObjKind, Syntax};
    if let AnimObjKind::CodeWindow {
        source_code,
        font_family,
        font_size,
        syntax,
        theme,
        title,
        title_font_size,
        width,
        height,
        background_color,
        code_id,
        close_btn_id,
        minimize_btn_id,
        maximize_btn_id,
        title_id,
        container_id,
        title_bar_bg_id,
        show_line_numbers,
        line_number_color,
        ..
    } = obj.kind
    {
        let syn = match syntax {
            Syntax::Rust => crate::anim_object::text::code::Syntax::Rust,
            Syntax::Nix => crate::anim_object::text::code::Syntax::Nix,
            Syntax::Python => crate::anim_object::text::code::Syntax::Python,
            Syntax::JS => crate::anim_object::text::code::Syntax::JS,
            Syntax::Zig => crate::anim_object::text::code::Syntax::Zig,
        };
        let t = theme.as_ref().unwrap_or(default_theme);
        let render_base16 = convert_base16(&t.base);
        let th = crate::anim_object::text::code::theme::Theme::from_base16(render_base16);
        let cw = code_window(
            obj.transform.position,
            source_code,
            th,
            font_family,
            Align::Left,
            font_size,
            syn,
            title,
            width,
            height,
            title_font_size,
            c(background_color),
            code_id,
            close_btn_id,
            minimize_btn_id,
            maximize_btn_id,
            title_id,
            obj.id,
            container_id,
            title_bar_bg_id,
            show_line_numbers,
            c(line_number_color),
        );
        Ok(cw.instantiate())
    } else {
        bail!("build_code_window_op called on non-CodeWindow kind")
    }
}

fn convert_core_anim_obj(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<crate::anim_object::object_trait::AnimObj> {
    use crate::anim_object::object_trait::AnimObj as RenderObj;
    let transform = make_transform(&obj);
    match obj.kind {
        scal_core::anim_obj::AnimObjKind::Rectangle {
            size,
            corner_radius,
            color,
        } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Rectangle {
                size,
                corner_radius,
                color: c(color),
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Circle { radius, color } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Circle {
                radius,
                color: c(color),
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Polygon {
            radius,
            sides,
            color,
        } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Polygon {
                radius,
                sides,
                color: c(color),
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Text {
            value,
            font_family,
            alignment,
            color,
            font_size,
        } => {
            let align = match alignment {
                scal_core::anim_obj::TextAlign::Center => crate::anim_object::text::Align::Center,
                scal_core::anim_obj::TextAlign::Left => crate::anim_object::text::Align::Left,
                scal_core::anim_obj::TextAlign::Right => crate::anim_object::text::Align::Right,
            };
            Ok(RenderObj(Box::new(crate::anim_object::text::Text {
                id: obj.id,
                value,
                font_family,
                alignment: align,
                color: c(color),
                font_size,
                transform,
                cached_size: None,
            })))
        }
        scal_core::anim_obj::AnimObjKind::Svg {
            path,
            size,
            tint,
            fill,
            stroke,
            stroke_width,
            stretch,
        } => {
            let st = match stretch {
                scal_core::anim_obj::StretchMode::Fit => {
                    crate::anim_object::image::StretchMode::Fit
                }
                scal_core::anim_obj::StretchMode::Fill => {
                    crate::anim_object::image::StretchMode::Fill
                }
            };
            Ok(RenderObj(Box::new(crate::anim_object::svg::Svg {
                path,
                size,
                tint: c(tint),
                fill: fill.map(c),
                stroke: stroke.map(c),
                stroke_width,
                stretch: st,
                transform,
            })))
        }
        scal_core::anim_obj::AnimObjKind::Image {
            path,
            size,
            color,
            stretch,
        } => {
            let st = match stretch {
                scal_core::anim_obj::StretchMode::Fit => {
                    crate::anim_object::image::StretchMode::Fit
                }
                scal_core::anim_obj::StretchMode::Fill => {
                    crate::anim_object::image::StretchMode::Fill
                }
            };
            Ok(RenderObj(Box::new(crate::anim_object::image::Image {
                path,
                size,
                color: c(color),
                stretch: st,
                transform,
            })))
        }
        scal_core::anim_obj::AnimObjKind::Code {
            source_code,
            font_family,
            font_size,
            syntax,
            theme,
            padding,
            show_line_numbers,
            line_number_color,
        } => {
            let syn = match syntax {
                scal_core::anim_obj::Syntax::Rust => crate::anim_object::text::code::Syntax::Rust,
                scal_core::anim_obj::Syntax::Nix => crate::anim_object::text::code::Syntax::Nix,
                scal_core::anim_obj::Syntax::Python => {
                    crate::anim_object::text::code::Syntax::Python
                }
                scal_core::anim_obj::Syntax::JS => crate::anim_object::text::code::Syntax::JS,
                scal_core::anim_obj::Syntax::Zig => crate::anim_object::text::code::Syntax::Zig,
            };
            let t = theme.as_ref().unwrap_or(default_theme);
            let render_base16 = convert_base16(&t.base);
            let th = crate::anim_object::text::code::theme::Theme::from_base16(render_base16);
            Ok(RenderObj(Box::new(crate::anim_object::text::code::Code {
                id: obj.id,
                source_code,
                theme: th,
                font_family,
                font_size,
                syntax: syn,
                padding,
                show_line_numbers,
                line_number_color: c(line_number_color),
                transform,
                alignment: crate::anim_object::text::Align::Left,
                lines: vec![],
                dirty: true,
                anim_reveal: 1.0,
                anim_spacing: 0.0,
                anim_line_start: 0,
                anim_line_end: 0,
                anim_style: crate::anim_object::text::code::CodeAnimationStyle::TypeWriter,
                anim_spacing_accum: 0.0,
                cached_size: None,
                highlights: vec![],
            })))
        }
        scal_core::anim_obj::AnimObjKind::CodeWindow { .. } => {
            bail!("CodeWindow should be handled by build_code_window_op")
        }
        scal_core::anim_obj::AnimObjKind::Group { .. } => {
            bail!("Group object conversion not yet implemented")
        }
    }
}
