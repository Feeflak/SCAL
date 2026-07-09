use std::collections::HashMap;

use anyhow::{Context, Result};
use wgpu::util::DeviceExt;
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CurrentSurfaceTexture, FragmentState, LoadOp, MultisampleState,
    Operations, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, StoreOp, Surface, SurfaceConfiguration, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    anim_object::{
        render::{get_pipelines, PipelineData, PipelineKind},
        text::render::TextRenderer,
    },
    anim_op::AnimOP,
    animator::Animator,
    renderer::draw_scene,
};

#[derive(Clone, Debug)]
pub struct TimelineOp {
    pub label: String,
    pub start_time: f32,
    pub end_time: f32,
    pub kind: OpKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    Instantiate,
    Transform,
    Code,
    Wait,
    Sound,
    Composite,
}

const UI_TIMELINE_HEIGHT: f32 = 48.0;
const UI_OPERATION_MARKER_HEIGHT: f32 = 16.0;
const UI_PLAYHEAD_WIDTH: f32 = 2.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIUniforms {
    resolution: [f32; 2],
}

fn op_label(op: &AnimOP) -> &'static str {
    match op {
        AnimOP::Instantiate(_) => "Instantiate",
        AnimOP::TransformMovePos(..) | AnimOP::TransformMoveToObj(..) => "Move",
        AnimOP::TransformRotate(..) => "Rotate",
        AnimOP::TransformScale(..) => "Scale",
        AnimOP::CodeAddLines(..) => "Code+",
        AnimOP::CodeModifyLine(..) => "Code~",
        AnimOP::CodeRemoveLines(..) => "Code-",
        AnimOP::CodeHighlight(..) => "Highlight",
        AnimOP::Current { .. } => "Snapshot",
        AnimOP::All(..) => "All",
        AnimOP::Sequence(..) => "Sequence",
        AnimOP::Wait(..) => "Wait",
        AnimOP::PlaySound(..) => "Sound",
    }
}

fn op_kind(op: &AnimOP) -> OpKind {
    match op {
        AnimOP::Instantiate(_) => OpKind::Instantiate,
        AnimOP::TransformMovePos(..) | AnimOP::TransformMoveToObj(..)
        | AnimOP::TransformRotate(..) | AnimOP::TransformScale(..) => OpKind::Transform,
        AnimOP::CodeAddLines(..) | AnimOP::CodeModifyLine(..)
        | AnimOP::CodeRemoveLines(..) | AnimOP::CodeHighlight(..) => OpKind::Code,
        AnimOP::Current { .. } => OpKind::Instantiate,
        AnimOP::All(..) | AnimOP::Sequence(..) => OpKind::Composite,
        AnimOP::Wait(..) => OpKind::Wait,
        AnimOP::PlaySound(..) => OpKind::Sound,
    }
}

fn op_color(kind: OpKind) -> [f32; 4] {
    match kind {
        OpKind::Instantiate => [0.2, 0.8, 0.2, 0.8],
        OpKind::Transform => [0.2, 0.4, 0.9, 0.8],
        OpKind::Code => [0.9, 0.7, 0.2, 0.8],
        OpKind::Wait => [0.4, 0.4, 0.4, 0.5],
        OpKind::Sound => [0.9, 0.2, 0.2, 0.8],
        OpKind::Composite => [0.6, 0.6, 0.8, 0.5],
    }
}

pub fn flatten_ops(ops: &[AnimOP]) -> (Vec<TimelineOp>, f32) {
    fn flatten_inner(ops: &[AnimOP], start_time: f32, result: &mut Vec<TimelineOp>) -> f32 {
        let mut time = start_time;
        for op in ops {
            let op_start = time;
            match op {
                AnimOP::All(children) => {
                    let mut max_end = time;
                    for child in children {
                        let end = flatten_inner(std::slice::from_ref(child), time, result);
                        if end > max_end {
                            max_end = end;
                        }
                    }
                    time = max_end;
                }
                AnimOP::Sequence(children) => {
                    time = flatten_inner(children, time, result);
                }
                AnimOP::Wait(d) => time += d,
                AnimOP::TransformMovePos(_, _, d, _)
                | AnimOP::TransformMoveToObj(_, _, _, d, _)
                | AnimOP::TransformRotate(_, _, d, _)
                | AnimOP::TransformScale(_, _, d, _) => time += d,
                AnimOP::CodeAddLines(_, _, _, d, _, _)
                | AnimOP::CodeModifyLine(_, _, _, d, _, _)
                | AnimOP::CodeRemoveLines(_, _, d, _, _) => time += d,
                AnimOP::CodeHighlight(_, action) => {
                    time += action.duration_and_curve().0;
                }
                AnimOP::PlaySound(_, _) => {}
                AnimOP::Instantiate(_) | AnimOP::Current { .. } => {}
            }
            if time > op_start {
                result.push(TimelineOp {
                    label: op_label(op).to_string(),
                    start_time: op_start,
                    end_time: time,
                    kind: op_kind(op),
                });
            }
        }
        time
    }
    let mut ops_list = vec![];
    let total = flatten_inner(ops, 0.0, &mut ops_list);
    (ops_list, total)
}

pub struct PreviewRenderer {
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,

    pipelines: HashMap<PipelineKind, PipelineData>,
    text_renderer: TextRenderer,
    renderer: crate::renderer::Renderer,
    animator: Animator,
    camera: crate::projection::Camera,
    background_color: crate::types::Color,
    text_resolution_multiplier: f32,

    current_frame: u64,
    total_frames: u64,
    fps: u32,
    paused: bool,
    finished: bool,
    frame_rendered: bool,

    timeline_ops: Vec<TimelineOp>,
    original_animations: Vec<AnimOP>,

    ui_pipeline: RenderPipeline,
    ui_bind_group_layout: wgpu::BindGroupLayout,
    ui_uniform_buffer: wgpu::Buffer,
    ui_vertex_buffer: Option<wgpu::Buffer>,
    ui_index_buffer: Option<wgpu::Buffer>,
    ui_index_count: u32,
}

impl PreviewRenderer {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
        surface: Surface<'static>,
        surface_config: SurfaceConfiguration,
        camera: crate::projection::Camera,
        background_color: crate::types::Color,
        animations: Vec<AnimOP>,
        fps: u32,
        text_resolution_multiplier: f32,
    ) -> Result<Self> {
        let (timeline_ops, total_dur) = flatten_ops(&animations);
        let total_frames = if total_dur > 0.0 {
            (total_dur * fps as f32).ceil() as u64
        } else {
            1
        };

        let text_renderer = TextRenderer::new(&device, text_resolution_multiplier);
        let mut pipelines = get_pipelines(&device, 1);
        pipelines
            .get_mut(&PipelineKind::Text)
            .expect("text pipeline missing")
            .bind_groups
            .push(text_renderer.bind_group.clone());

        let animator = Animator::new(
            animations.clone(),
            fps,
            camera,
            text_resolution_multiplier,
        )?;

        let renderer = crate::renderer::Renderer::new(&device);

        let (ui_pipeline, ui_bind_group_layout, ui_uniform_buffer) =
            Self::create_ui_pipeline(&device)?;

        Ok(PreviewRenderer {
            device,
            queue,
            surface,
            surface_config,
            pipelines,
            text_renderer,
            renderer,
            animator,
            camera,
            background_color,
            text_resolution_multiplier,
            current_frame: 0,
            total_frames,
            fps,
            paused: false,
            finished: false,
            frame_rendered: false,
            timeline_ops,
            original_animations: animations,
            ui_pipeline,
            ui_bind_group_layout,
            ui_uniform_buffer,
            ui_vertex_buffer: None,
            ui_index_buffer: None,
            ui_index_count: 0,
        })
    }

    fn create_ui_pipeline(
        device: &wgpu::Device,
    ) -> Result<(
        RenderPipeline,
        wgpu::BindGroupLayout,
        wgpu::Buffer,
    )> {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("ui shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                include_str!("preview_ui.wgsl"),
            )),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ui bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<UIUniforms>() as u64,
                        ),
                    },
                    count: None,
                }],
            },
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("ui pipeline layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            },
        );

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ui pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<UIVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui uniform buffer"),
            size: std::mem::size_of::<UIUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok((pipeline, bind_group_layout, uniform_buffer))
    }

    fn build_ui_geometry(&mut self) {
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        if width == 0 || height == 0 {
            return;
        }

        let w = width as f32;
        let h = height as f32;

        let timeline_top = h - UI_TIMELINE_HEIGHT;
        let timeline_bottom = h;

        let total_dur = if self.total_frames > 0 {
            self.total_frames as f32 / self.fps as f32
        } else {
            1.0
        };

        let current_t = if total_dur > 0.0 {
            (self.current_frame as f32 / self.fps as f32) / total_dur
        } else {
            0.0
        };
        let playhead_x = current_t * w;

        let mut vertices: Vec<UIVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Timeline background bar
        let bg_color = [0.05, 0.05, 0.1, 0.85];
        append_rect(&mut vertices, &mut indices, 0.0, timeline_top, w, timeline_bottom, bg_color);

        // Operation markers
        for op in &self.timeline_ops {
            let op_start_x = (op.start_time / total_dur) * w;
            let op_end_x = (op.end_time / total_dur) * w;
            let op_w = (op_end_x - op_start_x).max(4.0);
            let color = op_color(op.kind);
            let marker_top = timeline_top + 4.0;
            let marker_bot = marker_top + UI_OPERATION_MARKER_HEIGHT;
            append_rect(
                &mut vertices,
                &mut indices,
                op_start_x,
                marker_top,
                op_start_x + op_w,
                marker_bot,
                color,
            );
        }

        // Playhead
        let ph_color = [1.0, 1.0, 1.0, 0.9];
        append_rect(
            &mut vertices,
            &mut indices,
            (playhead_x - UI_PLAYHEAD_WIDTH / 2.0).max(0.0),
            timeline_top,
            (playhead_x + UI_PLAYHEAD_WIDTH / 2.0).min(w),
            timeline_bottom,
            ph_color,
        );

        self.ui_index_count = indices.len() as u32;

        if vertices.is_empty() {
            self.ui_vertex_buffer = None;
            self.ui_index_buffer = None;
            return;
        }

        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ui vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ui index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        self.ui_vertex_buffer = Some(vertex_buffer);
        self.ui_index_buffer = Some(index_buffer);
    }

    /// Advance the animator by one frame and render + present.
    /// Returns false when the animation is finished.
    pub fn advance_render_present(&mut self) -> Result<bool> {
        if self.finished {
            return Ok(false);
        }

        let have_scene = if !self.paused && self.current_frame < self.total_frames {
            // Advance the animator
            let frame_data = self.animator.animate_next_frame()
                .context("while advancing animation frame")?;

            match frame_data {
                Some(data) => {
                    self.current_frame += 1;
                    if self.current_frame >= self.total_frames {
                        self.finished = true;
                    }

                    // Update glyphs
                    if let Some(glyph_data) = data.glyph_update_data {
                        self.text_renderer
                            .update_glyphs_if_needed(glyph_data, &self.queue);
                    }

                    let scene = data.scene;

                    // Acquire surface texture
                    let frame = match self.surface.get_current_texture() {
                        CurrentSurfaceTexture::Success(t)
                        | CurrentSurfaceTexture::Suboptimal(t) => t,
                        CurrentSurfaceTexture::Timeout
                        | CurrentSurfaceTexture::Occluded => {
                            // Skip this frame
                            self.frame_rendered = true;
                            return Ok(true);
                        }
                        CurrentSurfaceTexture::Outdated => {
                            self.surface.configure(&self.device, &self.surface_config);
                            self.frame_rendered = true;
                            return Ok(true);
                        }
                        CurrentSurfaceTexture::Lost | CurrentSurfaceTexture::Validation => {
                            log::warn!("Surface lost, skipping frame");
                            self.frame_rendered = true;
                            return Ok(true);
                        }
                    };
                    let surface_view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = self.device
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Preview Frame Encoder"),
                        });

                    // Render scene
                    {
                        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: Some("Scene Pass"),
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &surface_view,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(wgpu::Color {
                                        r: self.background_color.r as f64,
                                        g: self.background_color.g as f64,
                                        b: self.background_color.b as f64,
                                        a: self.background_color.a as f64,
                                    }),
                                    store: StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                        draw_scene(
                            &mut rpass,
                            &self.pipelines,
                            &self.device,
                            &self.queue,
                            scene,
                            &mut self.renderer,
                        )
                        .context("while drawing scene")?;
                    }

                    // Render UI overlay
                    self.build_ui_geometry();
                    if self.ui_index_count > 0 {
                        self.render_ui_pass(&mut encoder, &surface_view);
                    }

                    self.queue.submit(Some(encoder.finish()));
                    frame.present();
                    self.frame_rendered = true;
                    true
                }
                None => {
                    self.finished = true;
                    false
                }
            }
        } else {
            // Paused or at end - still need to render current state
            // Only render if we haven't rendered this frame yet
            if !self.frame_rendered {
                self.frame_rendered = true;

                let frame = match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t)
                    | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => {
                        return Ok(true);
                    }
                };
                let surface_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("Preview Frame Encoder"),
                    });

                // Just clear the screen and show UI
                {
                    let _rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Clear Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &surface_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color {
                                    r: self.background_color.r as f64,
                                    g: self.background_color.g as f64,
                                    b: self.background_color.b as f64,
                                    a: self.background_color.a as f64,
                                }),
                                store: StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }

                // UI only
                self.build_ui_geometry();
                if self.ui_index_count > 0 {
                    self.render_ui_pass(&mut encoder, &surface_view);
                }

                self.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            true
        };

        if !have_scene && !self.paused {
            return Ok(false);
        }
        Ok(true)
    }

    fn render_ui_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let Some(ref vertex_buffer) = self.ui_vertex_buffer else { return };
        let Some(ref index_buffer) = self.ui_index_buffer else { return };
        if self.ui_index_count == 0 {
            return;
        }

        // Update uniforms
        let uniforms = UIUniforms {
            resolution: [self.surface_config.width as f32, self.surface_config.height as f32],
        };
        self.queue.write_buffer(&self.ui_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = self
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ui bind group"),
                layout: &self.ui_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.ui_uniform_buffer.as_entire_binding(),
                }],
            });

        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("UI Overlay Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.ui_pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.ui_index_count, 0, 0..1);
    }

    pub fn seek_to(&mut self, time_sec: f32) -> Result<()> {
        let frame = (time_sec * self.fps as f32).round() as u64;
        self.current_frame = frame.min(self.total_frames);
        self.finished = false;
        self.frame_rendered = false;

        let mut animator = Animator::new(
            self.original_animations.clone(),
            self.fps,
            self.camera,
            self.text_resolution_multiplier,
        )?;

        // Fast-forward
        for _ in 0..self.current_frame {
            if animator.animate_next_frame()?.is_none() {
                self.finished = true;
                break;
            }
        }

        self.animator = animator;
        Ok(())
    }

    pub fn current_time(&self) -> f32 {
        if self.total_frames > 0 {
            self.current_frame as f32 / self.fps as f32
        } else {
            0.0
        }
    }

    pub fn total_duration(&self) -> f32 {
        if self.total_frames > 0 {
            self.total_frames as f32 / self.fps as f32
        } else {
            0.0
        }
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.frame_rendered = false;
    }

    pub fn set_paused(&mut self, paused: bool) {
        let was = self.paused;
        self.paused = paused;
        if was != paused {
            self.frame_rendered = false;
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn replay(&mut self) -> Result<()> {
        self.seek_to(0.0)
    }

    pub fn step_forward(&mut self) -> Result<bool> {
        self.paused = true;
        if self.current_frame < self.total_frames {
            self.advance_render_present()
        } else {
            Ok(false)
        }
    }

    pub fn step_backward(&mut self) -> Result<()> {
        self.paused = true;
        self.frame_rendered = false;
        if self.current_frame > 0 {
            let target = self.current_frame.saturating_sub(2); // -2 because current starts at 1 after advance
            self.seek_to((target as f32 / self.fps as f32).max(0.0))?;
        } else {
            self.seek_to(0.0)?;
        }
        Ok(())
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn reload(&mut self, animations: Vec<AnimOP>) -> Result<()> {
        let (timeline_ops, total_dur) = flatten_ops(&animations);
        self.total_frames = if total_dur > 0.0 {
            (total_dur * self.fps as f32).ceil() as u64
        } else {
            1
        };
        self.timeline_ops = timeline_ops;
        self.original_animations = animations;
        self.current_frame = 0;
        self.finished = false;
        self.paused = false;
        self.frame_rendered = false;

        self.animator = Animator::new(
            self.original_animations.clone(),
            self.fps,
            self.camera,
            self.text_resolution_multiplier,
        )?;

        Ok(())
    }

    pub fn timeline_ops(&self) -> &[TimelineOp] {
        &self.timeline_ops
    }

    /// Get the current animator state. Note: when paused, the scene may not
    /// reflect the latest frame since animate_next_frame was already called.
    /// Use this for reading current time info rather than scene rendering.
    pub fn animator(&self) -> &Animator {
        &self.animator
    }
}

fn append_rect(
    vertices: &mut Vec<UIVertex>,
    indices: &mut Vec<u32>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: [f32; 4],
) {
    let base = vertices.len() as u32;
    vertices.push(UIVertex { position: [x0, y0], color });
    vertices.push(UIVertex { position: [x1, y0], color });
    vertices.push(UIVertex { position: [x1, y1], color });
    vertices.push(UIVertex { position: [x0, y1], color });
    indices.push(base);
    indices.push(base + 1);
    indices.push(base + 2);
    indices.push(base);
    indices.push(base + 2);
    indices.push(base + 3);
}
