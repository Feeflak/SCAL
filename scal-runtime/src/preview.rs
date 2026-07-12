use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use notify::{EventKind, RecursiveMode, Watcher};
use scal_core::{Camera, Color};
use tokio::sync::watch;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use winit::platform::wayland::EventLoopBuilderExtWayland;

use crate::Config;
use crate::anim_object::render::{PipelineData, PipelineKind, get_pipelines};
use crate::anim_object::text::render::TextRenderer;
use crate::anim_op::AnimOperation;
use crate::animator::Animator;
use crate::conversion::convert_anim_ops;
use crate::renderer::draw_scene;

pub async fn run_preview(config: Config) -> Result<()> {
    let (project, default_theme) = load_animation(&config.animation.binary).await?;

    let win_scale = 0.5;
    let win_w = (config.rendering.width as f64 * win_scale) as u32;
    let win_h = (config.rendering.height as f64 * win_scale) as u32;

    let (reload_tx, reload_rx) = watch::channel(false);
    let watch_dir = std::env::current_dir().context("Failed to get current dir")?;

    let _watcher_handle = start_file_watcher(watch_dir, reload_tx);

    let th_config = AnimationConfig {
        win_w: win_w.max(100),
        win_h: win_h.max(100),
        fps: config.rendering.fps,
        text_mult: config.rendering.text_resolution_multiplier,
        background_color: Color::new(
            project.scene_settings.background_color.r,
            project.scene_settings.background_color.g,
            project.scene_settings.background_color.b,
            project.scene_settings.background_color.a,
        ),
        camera: Camera::new(
            project.scene_settings.camera.virtual_size,
            project.scene_settings.camera.position,
            project.scene_settings.camera.zoom,
        ),
        animations: convert_anim_ops(project.timeline, &project.scene_settings.default_theme)?,
        binary: config.animation.binary.clone(),
        default_theme,
    };

    run_event_loop(th_config, reload_rx)
}

struct AnimationConfig {
    win_w: u32,
    win_h: u32,
    fps: u32,
    text_mult: f32,
    background_color: Color,
    camera: Camera,
    animations: Vec<AnimOperation>,
    binary: String,
    default_theme: scal_core::Theme,
}

fn run_event_loop(config: AnimationConfig, reload_rx: watch::Receiver<bool>) -> Result<()> {
    let mut builder = EventLoop::builder();

    #[cfg(any(feature = "wayland"))]
    {
        use winit::platform::wayland::EventLoopBuilderExtWayland;
        EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    }

    #[cfg(any(feature = "x11"))]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
    }

    let event_loop = builder
        .with_any_thread(true)
        .build()
        .context("Failed to create event loop")?;

    let mut state = PreviewState {
        config: Some(config),
        preview: None,
        window: None,
        window_size: (0, 0),
        needs_render: true,
        reload_requested: false,
        cursor_pos: (0.0, 0.0),
        mouse_down: false,
        last_seek_time: std::time::Instant::now(),
        last_frame_advance: std::time::Instant::now(),
        modifiers: None,
        reload_rx: reload_rx.clone(),
        device: None,
        queue: None,
        instance: None,
        binary: None,
        default_theme: None,
    };

    event_loop
        .run_app(&mut state)
        .map_err(|e| anyhow::anyhow!("Event loop error: {e}"))
}

struct PreviewState {
    config: Option<AnimationConfig>,
    preview: Option<PreviewRenderer>,
    window: Option<Arc<Window>>,
    window_size: (u32, u32),
    needs_render: bool,
    reload_requested: bool,
    cursor_pos: (f64, f64),
    mouse_down: bool,
    last_seek_time: std::time::Instant,
    last_frame_advance: std::time::Instant,
    modifiers: Option<winit::event::Modifiers>,
    reload_rx: watch::Receiver<bool>,
    device: Option<std::sync::Arc<wgpu::Device>>,
    queue: Option<std::sync::Arc<wgpu::Queue>>,
    instance: Option<wgpu::Instance>,
    binary: Option<String>,
    default_theme: Option<scal_core::Theme>,
}

impl ApplicationHandler for PreviewState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.preview.is_some() {
            return;
        }

        let config = self.config.take().expect("config already consumed");

        let window_attrs = Window::default_attributes()
            .with_title("SCAL Preview")
            .with_inner_size(winit::dpi::LogicalSize::new(
                config.win_w as f64,
                config.win_h as f64,
            ));
        let window = event_loop
            .create_window(window_attrs)
            .expect("Failed to create window");
        let window = Arc::new(window);

        self.window_size = (config.win_w, config.win_h);

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create wgpu surface");
        // Extend surface lifetime: safe because window (Arc) outlives the surface
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .context("Failed to request wgpu adapter")
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("preview device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::default(),
        }))
        .expect("Failed to request wgpu device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| **f == wgpu::TextureFormat::Rgba8Unorm)
            .copied()
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: config.win_w.max(1),
            height: config.win_h.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let preview = pollster::block_on(PreviewRenderer::new(
            device.clone(),
            queue.clone(),
            surface,
            surface_config,
            config.camera,
            config.background_color,
            config.animations,
            config.fps,
            config.text_mult,
        ))
        .expect("Failed to create preview renderer");

        self.preview = Some(preview);
        self.window = Some(window);
        self.device = Some(device);
        self.queue = Some(queue);
        self.instance = Some(instance);
        self.binary = Some(config.binary);
        self.default_theme = Some(config.default_theme);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut preview) = self.preview else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                _event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let w = size.width.max(1);
                let h = size.height.max(1);
                self.window_size = (w, h);
                preview.handle_resize(w, h);
                self.needs_render = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = Some(modifiers);
            }
            WindowEvent::RedrawRequested => {
                // Check for reload
                if self.reload_requested {
                    self.reload_requested = false;
                    if let (Some(binary), Some(theme)) =
                        (self.binary.as_ref(), self.default_theme.as_ref())
                    {
                        let result = reload_animation(binary, theme, preview);
                        if let Err(e) = result {
                            log::error!("Reload failed: {e}");
                        }
                    }
                }

                // Clamp advance rate to the configured FPS so animation timing
                // matches wall clock (and stays in sync with audio).
                let should_advance = if preview.is_finished() || preview.is_paused() {
                    true
                } else {
                    let now = std::time::Instant::now();
                    let frame_dt = std::time::Duration::from_secs_f64(1.0 / preview.fps() as f64);
                    now - self.last_frame_advance >= frame_dt
                };

                if should_advance {
                    self.last_frame_advance = std::time::Instant::now();
                    match preview.advance_render_present() {
                        Ok(more) => {
                            if !more {
                                // Replay on finish
                                let _ = preview.replay();
                            }
                        }
                        Err(e) => {
                            log::error!("Render error: {e:#}");
                        }
                    }
                }
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let ctrl = self.modifiers.as_ref().map_or(false, |m| {
                    m.state()
                        .intersects(winit::keyboard::ModifiersState::CONTROL)
                });
                if ctrl {
                    let y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 10.0,
                    };
                    if y > 0.0 {
                        preview.zoom_in();
                    } else if y < 0.0 {
                        preview.zoom_out();
                    }
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { ref event, .. }
                if event.state == winit::event::ElementState::Pressed =>
            {
                handle_keyboard(event, preview, self.modifiers);
                self.needs_render = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
                preview.set_hovered_from_cursor(position.x as f32, position.y as f32);
                if !preview.frame_rendered {
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
                if self.mouse_down
                    && self.last_seek_time.elapsed() >= std::time::Duration::from_millis(33)
                    && seek_on_timeline(preview, self.window_size, self.cursor_pos)
                {
                    self.last_seek_time = std::time::Instant::now();
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                self.mouse_down = true;
                let was_on_timeline = seek_on_timeline(preview, self.window_size, self.cursor_pos);
                let (cx, cy) = self.cursor_pos;
                let (ww, wh) = self.window_size;
                let h = wh as f32;
                let timeline_top = h - 80.0;
                let marker_bot = h - 34.0;
                let op_click = if cy as f32 >= timeline_top && cy as f32 <= marker_bot {
                    preview.find_op_at_x(cx as f32).map(|op| {
                        let info = match op.source_loc {
                            Some(ref loc) => {
                                format!("{} @ {}:{}", op.label, loc.file, loc.line)
                            }
                            None => format!("{} (no source)", op.label),
                        };
                        log::info!("Op: {}", info);
                        let file = op
                            .source_loc
                            .as_ref()
                            .map(|l| l.file.clone())
                            .unwrap_or_default();
                        let line = op.source_loc.as_ref().map(|l| l.line).unwrap_or(0);
                        let label = op.label.clone();
                        (info, file, line, label)
                    })
                } else if cy as f32 >= marker_bot && cy as f32 <= h {
                    preview
                        .find_sound_at_x(cx as f32)
                        .map(|(start, end, _idx, marker)| {
                            let end_str = end.map_or("?".to_string(), |e| format!("{:.2}s", e));
                            let info = format!("{} @ {}:{}", marker.name, marker.file, marker.line);
                            log::info!("Sound: {}", info);
                            (info, marker.file.clone(), marker.line, marker.name.clone())
                        })
                } else {
                    None
                };
                if let Some((ref info, ref file, line, ref label)) = op_click {
                    preview.set_selected_op(Some((file.as_str(), line, label.as_str())));
                    if let Some(ref window) = self.window {
                        window.set_title(&format!("SCAL Preview - {}", info));
                        window.request_redraw();
                    }
                } else if was_on_timeline {
                    preview.set_selected_op(None);
                    if let Some(ref window) = self.window {
                        window.set_title("SCAL Preview");
                        window.request_redraw();
                    }
                } else {
                    preview.set_selected_op(None);
                    preview.toggle_pause();
                    if let Some(ref window) = self.window {
                        window.set_title("SCAL Preview");
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                self.mouse_down = false;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.reload_rx.has_changed().unwrap_or(false) {
            self.reload_requested = true;
        }
        // Mark the change as seen so has_changed() doesn't keep returning true
        // on every frame, which would cause an infinite reload loop.
        self.reload_rx.borrow_and_update();
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn seek_on_timeline(
    preview: &mut PreviewRenderer,
    window_size: (u32, u32),
    cursor_pos: (f64, f64),
) -> bool {
    let (ww, wh) = window_size;
    if ww > 0 && wh > 0 {
        let (cx, cy) = cursor_pos;
        let timeline_top = wh as f64 - 80.0;
        if cy >= timeline_top && cy <= wh as f64 {
            let seek_time = preview.screen_x_to_time(cx as f32);
            if let Err(e) = preview.seek_to(seek_time) {
                log::error!("Seek error: {e}");
            }
            return true;
        }
    }
    false
}

fn seek_by_seconds(preview: &mut PreviewRenderer, offset: f32) {
    let new_time = (preview.current_time() + offset).clamp(0.0, preview.total_duration());
    if let Err(e) = preview.seek_to(new_time) {
        log::error!("Seek error: {e}");
    }
}

fn handle_keyboard(
    key_event: &winit::event::KeyEvent,
    preview: &mut PreviewRenderer,
    modifiers: Option<winit::event::Modifiers>,
) {
    let shift = modifiers.as_ref().map_or(false, |m| {
        m.state().intersects(winit::keyboard::ModifiersState::SHIFT)
    });
    match &key_event.logical_key {
        Key::Named(NamedKey::Space) => {
            let was_paused = preview.is_paused();
            preview.toggle_pause();
            log::info!("{}", if was_paused { "Playing" } else { "Paused" });
        }
        Key::Named(NamedKey::ArrowRight) if shift => seek_by_seconds(preview, 1.0),
        Key::Named(NamedKey::ArrowRight) => {
            if let Err(e) = preview.step_forward() {
                log::error!("Step forward error: {e}");
            }
        }
        Key::Named(NamedKey::ArrowLeft) if shift => seek_by_seconds(preview, -1.0),
        Key::Named(NamedKey::ArrowLeft) => {
            if let Err(e) = preview.step_backward() {
                log::error!("Step backward error: {e}");
            }
        }
        Key::Character(c) if (c == "l" || c == "L") && shift => seek_by_seconds(preview, 1.0),
        Key::Character(c) if c == "l" || c == "L" => {
            if let Err(e) = preview.step_forward() {
                log::error!("Step forward error: {e}");
            }
        }
        Key::Character(c) if (c == "h" || c == "H") && shift => seek_by_seconds(preview, -1.0),
        Key::Character(c) if c == "h" || c == "H" => {
            if let Err(e) = preview.step_backward() {
                log::error!("Step backward error: {e}");
            }
        }
        Key::Named(NamedKey::ArrowUp) => {
            let new_time = (preview.current_time() + 5.0).min(preview.total_duration());
            if let Err(e) = preview.seek_to(new_time) {
                log::error!("Seek error: {e}");
            }
        }
        Key::Named(NamedKey::ArrowDown) => {
            let new_time = (preview.current_time() - 5.0).max(0.0);
            if let Err(e) = preview.seek_to(new_time) {
                log::error!("Seek error: {e}");
            }
        }
        Key::Named(NamedKey::Home) => {
            if let Err(e) = preview.replay() {
                log::error!("Replay error: {e}");
            }
        }
        Key::Named(NamedKey::End) => {
            if let Err(e) = preview.seek_to(preview.total_duration()) {
                log::error!("Seek error: {e}");
            }
        }
        Key::Character(c) if c == "j" || c == "J" => {
            preview.zoom_in();
        }
        Key::Character(c) if c == "k" || c == "K" => {
            preview.zoom_out();
        }
        Key::Character(c) if c == "r" || c == "R" => {
            // Manual reload triggered by watcher
        }
        _ => {}
    }
}

async fn load_animation(binary: &str) -> Result<(scal_core::Project, scal_core::Theme)> {
    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(binary)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("Failed to spawn animation binary")?;

    let output = child
        .wait_with_output()
        .context("Failed to wait for animation binary")?;

    if !output.status.success() {
        bail!("Animation binary exited with error: {}", output.status);
    }

    let data = output.stdout;

    if data.len() < 8 {
        bail!("No data received from animation binary");
    }

    let (len_bytes, rest) = data.split_at(8);
    let len = usize::try_from(u64::from_le_bytes(len_bytes.try_into()?))?;

    if rest.len() < len {
        bail!("Incomplete project data received");
    }

    let project: scal_core::Project =
        bincode::deserialize(&rest[..len]).context("Failed to deserialize project")?;

    let theme = project.scene_settings.default_theme.clone();

    log::info!(
        "Loaded project with {} timeline operations",
        project.timeline.len()
    );

    Ok((project, theme))
}

fn reload_animation(
    binary: &str,
    default_theme: &scal_core::Theme,
    preview: &mut PreviewRenderer,
) -> Result<()> {
    log::info!("Reloading animation...");

    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(binary)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("Failed to spawn animation binary")?;

    let output = child
        .wait_with_output()
        .context("Failed to wait for animation binary")?;

    if !output.status.success() {
        log::warn!("Animation binary exited with error: {}", output.status);
        return Ok(());
    }

    let data = output.stdout;
    if data.len() < 8 {
        log::warn!("No data received from animation binary");
        return Ok(());
    }

    let (len_bytes, rest) = data.split_at(8);
    let len = usize::try_from(u64::from_le_bytes(len_bytes.try_into()?))?;
    if rest.len() < len {
        log::warn!("Incomplete project data received");
        return Ok(());
    }

    let project: scal_core::Project =
        bincode::deserialize(&rest[..len]).context("Failed to deserialize project")?;

    let render_animations = convert_anim_ops(project.timeline, default_theme)?;
    preview.reload(render_animations)?;

    log::info!("Animation reloaded successfully");
    Ok(())
}

fn start_file_watcher(
    watch_dir: PathBuf,
    reload_tx: watch::Sender<bool>,
) -> Result<std::thread::JoinHandle<()>> {
    let handle = std::thread::Builder::new()
        .name("file-watcher".into())
        .spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel::<bool>();

            let mut watcher =
                match notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                    if let Ok(event) = event {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                                for path in &event.paths {
                                    if path.components().any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git") {
                                        continue;
                                    }
                                    if path.extension().map_or(false, |ext| ext == "rs") {
                                        let _ = tx.send(true);
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }) {
                    Ok(w) => w,
                    Err(e) => {
                        log::warn!("Failed to create file watcher: {e}");
                        return;
                    }
                };

            if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::Recursive) {
                log::warn!("Failed to start watching {watch_dir:?}: {e}");
                return;
            }

            let mut last_reload = std::time::Instant::now();
            while rx.recv().is_ok() {
                let now = std::time::Instant::now();
                if now.duration_since(last_reload) > Duration::from_millis(500) {
                    last_reload = now;
                    log::info!("File change detected, scheduling reload...");
                    let _ = reload_tx.send(true);
                }
            }
        })
        .context("Failed to spawn file watcher thread")?;

    Ok(handle)
}

use std::collections::HashMap;

use crate::audio_player::AudioPlayer;
use crate::sfx::{AudioEngine, collect_sounds_from_ops, compute_waveform};
use cosmic_text::{Attrs, FontSystem, Metrics, Shaping, SwashCache};
use wgpu::util::DeviceExt;
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CurrentSurfaceTexture, FragmentState, LoadOp, MultisampleState,
    Operations, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StoreOp,
    Surface, SurfaceConfiguration, TextureFormat, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};

#[derive(Clone, Debug)]
pub struct TimelineOp {
    pub label: String,
    pub start_time: f32,
    pub end_time: f32,
    pub kind: OpKind,
    pub source_loc: Option<scal_core::SourceLoc>,
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

#[derive(Clone, Debug)]
pub struct SoundMarker {
    pub start: f32,
    pub end: Option<f32>,
    pub file: String,
    pub line: u32,
    pub name: String,
}

const UI_TIMELINE_HEIGHT: f32 = 80.0;

const UI_OPERATION_MARKER_HEIGHT: f32 = 20.0;
const UI_PLAYHEAD_WIDTH: f32 = 2.0;

const UI_TIME_TEXT_FONT_SIZE: f32 = 17.0;
const UI_TIME_TEXT_HEIGHT: f32 = 22.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIUniforms {
    resolution: [f32; 2],
}

fn op_label(op: &AnimOperation) -> &'static str {
    match op {
        AnimOperation::Instantiate(..) => "Instantiate",
        AnimOperation::TransformMovePos(..) | AnimOperation::TransformMoveToObj(..) => "Move",
        AnimOperation::TransformRotate(..) => "Rotate",
        AnimOperation::TransformScale(..) => "Scale",
        AnimOperation::CodeAddLines(..) => "Add Lines",
        AnimOperation::CodeModifyLine(..) => "Modify Line",
        AnimOperation::CodeRemoveLines(..) => "Remove Lines",
        AnimOperation::CodeHighlight(..) => "Highlight",
        AnimOperation::All(..) => "All",
        AnimOperation::Sequence(..) => "Sequence",
        AnimOperation::Wait(..) => "Wait",
        AnimOperation::PlaySound(..) => "Sound",
    }
}

fn op_kind(op: &AnimOperation) -> OpKind {
    match op {
        AnimOperation::Instantiate(..) => OpKind::Instantiate,
        AnimOperation::TransformMovePos(..)
        | AnimOperation::TransformMoveToObj(..)
        | AnimOperation::TransformRotate(..)
        | AnimOperation::TransformScale(..) => OpKind::Transform,
        AnimOperation::CodeAddLines(..)
        | AnimOperation::CodeModifyLine(..)
        | AnimOperation::CodeRemoveLines(..)
        | AnimOperation::CodeHighlight(..) => OpKind::Code,
        AnimOperation::All(..) | AnimOperation::Sequence(..) => OpKind::Composite,
        AnimOperation::Wait(..) => OpKind::Wait,
        AnimOperation::PlaySound(..) => OpKind::Sound,
    }
}

fn op_color(kind: OpKind) -> [f32; 4] {
    match kind {
        OpKind::Instantiate => [0.2, 0.8, 0.2, 1.0],
        OpKind::Transform => [0.2, 0.4, 0.9, 1.0],
        OpKind::Code => [0.9, 0.7, 0.2, 1.0],
        OpKind::Wait => [0.4, 0.4, 0.4, 1.0],
        OpKind::Sound => [0.9, 0.2, 0.2, 1.0],
        OpKind::Composite => [0.6, 0.6, 0.8, 1.0],
    }
}

fn op_source_loc(op: &AnimOperation) -> Option<scal_core::SourceLoc> {
    match op {
        AnimOperation::Instantiate(_, loc)
        | AnimOperation::TransformMovePos(_, _, _, _, loc)
        | AnimOperation::TransformMoveToObj(_, _, _, _, _, loc)
        | AnimOperation::TransformRotate(_, _, _, _, loc)
        | AnimOperation::TransformScale(_, _, _, _, loc)
        | AnimOperation::CodeAddLines(_, _, _, _, _, _, loc)
        | AnimOperation::CodeModifyLine(_, _, _, _, _, _, loc)
        | AnimOperation::CodeRemoveLines(_, _, _, _, _, loc)
        | AnimOperation::CodeHighlight(_, _, loc)
        | AnimOperation::All(_, loc)
        | AnimOperation::Sequence(_, loc)
        | AnimOperation::Wait(_, loc)
        | AnimOperation::PlaySound(_, _, loc) => loc.clone(),
    }
}

pub fn flatten_ops(ops: &[AnimOperation]) -> (Vec<TimelineOp>, f32) {
    fn flatten_inner(ops: &[AnimOperation], start_time: f32, result: &mut Vec<TimelineOp>) -> f32 {
        let mut time = start_time;
        for op in ops {
            let op_start = time;
            match op {
                AnimOperation::All(children, _) => {
                    let mut max_end = time;
                    for child in children {
                        let end = flatten_inner(std::slice::from_ref(child), time, result);
                        if end > max_end {
                            max_end = end;
                        }
                    }
                    time = max_end;
                }
                AnimOperation::Sequence(children, _) => {
                    time = flatten_inner(children, time, result);
                }
                AnimOperation::Wait(d, _) => time += d,
                AnimOperation::TransformMovePos(_, _, d, _, _)
                | AnimOperation::TransformMoveToObj(_, _, _, d, _, _)
                | AnimOperation::TransformRotate(_, _, d, _, _)
                | AnimOperation::TransformScale(_, _, d, _, _) => time += d,
                AnimOperation::CodeAddLines(_, _, _, d, _, _, _)
                | AnimOperation::CodeModifyLine(_, _, _, d, _, _, _)
                | AnimOperation::CodeRemoveLines(_, _, d, _, _, _) => time += d,
                AnimOperation::CodeHighlight(_, action, _) => {
                    time += action.duration_and_curve().0;
                }
                AnimOperation::PlaySound(_, _, _) => {}
                AnimOperation::Instantiate(..) => {}
            }
            if time > op_start {
                result.push(TimelineOp {
                    label: op_label(op).to_string(),
                    start_time: op_start,
                    end_time: time,
                    kind: op_kind(op),
                    source_loc: op_source_loc(op),
                });
            }
        }
        time
    }
    let mut ops_list = vec![];
    let total = flatten_inner(ops, 0.0, &mut ops_list);
    (ops_list, total)
}

struct TimeTextLabel {
    time: f32,
    x_pixel: f32,
    uv_rect: [f32; 4],
}

struct TimeTextAtlas {
    font_system: FontSystem,
    swash_cache: SwashCache,
    labels: Vec<TimeTextLabel>,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    atlas_width: u32,
    atlas_height: u32,
    pixels: Vec<u8>,
    dirty: bool,
    overlay_texture: Option<wgpu::Texture>,
    overlay_bind_group: Option<wgpu::BindGroup>,
    overlay_width: f32,
    overlay_height: f32,
}

impl TimeTextAtlas {
    fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("time text atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        // Create a default 1x1 white texture as fallback
        let default_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("time text atlas default"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let default_view = default_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("time text bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let default_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("time text atlas default bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        TimeTextAtlas {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            labels: Vec::new(),
            texture: Some(default_tex),
            bind_group: Some(default_bg),
            bind_group_layout,
            sampler,
            atlas_width: 1,
            atlas_height: 1,
            pixels: Vec::new(),
            dirty: true,
            overlay_texture: None,
            overlay_bind_group: None,
            overlay_width: 0.0,
            overlay_height: 0.0,
        }
    }

    fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        let attrs = Attrs::new();
        buffer.set_text(
            text,
            &attrs,
            Shaping::Advanced,
            Some(cosmic_text::Align::Left),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let (width, total_height) = buffer.layout_runs().fold((0.0f32, 0.0f32), |(w, h), run| {
            (w.max(run.line_w), h + run.line_height)
        });
        (width, total_height)
    }

    fn rasterize_text(&mut self, text: &str, font_size: f32) -> (Vec<u8>, u32, u32) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        buffer.set_text(
            text,
            &Attrs::new(),
            Shaping::Advanced,
            Some(cosmic_text::Align::Left),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let (width_f, height_f) = self.measure_text(text, font_size);
        let w = width_f.ceil() as u32;
        let h = height_f.ceil() as u32;
        if w == 0 || h == 0 {
            return (vec![], 1, 1);
        }

        let mut canvas = vec![0u8; (w * h) as usize];

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0.0, run.line_y), 1.0);
                if let Some(image) = self
                    .swash_cache
                    .get_image(&mut self.font_system, physical.cache_key)
                {
                    let place = image.placement;
                    for row in 0..place.height as u32 {
                        for col in 0..place.width as u32 {
                            let src_idx = (row * place.width as u32 + col) as usize;
                            if src_idx < image.data.len() {
                                let alpha = image.data[src_idx];
                                let dx = (physical.x as i32 + col as i32) as i32;
                                let dy = (physical.y as i32 - place.top as i32 + row as i32) as i32;
                                if dx >= 0 && dy >= 0 && dx < w as i32 && dy < h as i32 {
                                    let dst_idx = (dy as u32 * w + dx as u32) as usize;
                                    if dst_idx < canvas.len() {
                                        canvas[dst_idx] = canvas[dst_idx].max(alpha);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (canvas, w, h)
    }

    fn rebuild(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        time_labels: &[(f32, String, f32)],
    ) {
        self.labels.clear();
        self.dirty = true;

        if time_labels.is_empty() {
            self.texture = None;
            self.bind_group = None;
            return;
        }

        const PADDING: u32 = 4;
        let mut total_w = PADDING;
        let max_h = UI_TIME_TEXT_HEIGHT as u32 + PADDING * 2;

        let mut rendered: Vec<(Vec<u8>, u32, u32)> = Vec::new();

        for &(_, ref text, _) in time_labels {
            let (canvas, w, h) = self.rasterize_text(text, UI_TIME_TEXT_FONT_SIZE);
            rendered.push((canvas, w.max(1), h.max(1)));
            total_w += w.max(1) + PADDING;
        }

        let atlas_w = total_w;
        let atlas_h = max_h;

        if atlas_w == 0 || atlas_h == 0 {
            self.texture = None;
            self.bind_group = None;
            return;
        }

        let mut atlas_pixels = vec![0u8; (atlas_w * atlas_h) as usize];
        let mut cursor_x = PADDING;
        let label_center_y = atlas_h / 2;

        for (i, &(ref time_val, _, _)) in time_labels.iter().enumerate() {
            let (ref canvas, label_w, label_h) = rendered[i];
            let y_offset = (label_center_y as i32 - (label_h / 2) as i32).max(0) as u32;

            for row in 0..label_h {
                for col in 0..label_w {
                    let src_idx = (row * label_w + col) as usize;
                    if src_idx < canvas.len() && canvas[src_idx] > 0 {
                        let dst_row = y_offset + row;
                        let dst_col = cursor_x + col;
                        if dst_row < atlas_h && dst_col < atlas_w {
                            let dst_idx = (dst_row * atlas_w + dst_col) as usize;
                            atlas_pixels[dst_idx] = atlas_pixels[dst_idx].max(canvas[src_idx]);
                        }
                    }
                }
            }

            let uv_x0 = cursor_x as f32 / atlas_w as f32;
            let uv_x1 = (cursor_x + label_w) as f32 / atlas_w as f32;
            let uv_y0 = y_offset as f32 / atlas_h as f32;
            let uv_y1 = (y_offset + label_h) as f32 / atlas_h as f32;

            self.labels.push(TimeTextLabel {
                time: *time_val,
                x_pixel: 0.0,
                uv_rect: [uv_x0, uv_y0, uv_x1, uv_y1],
            });

            cursor_x += label_w + PADDING;
        }

        self.atlas_width = atlas_w;
        self.atlas_height = atlas_h;
        self.pixels = atlas_pixels;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("time text atlas"),
            size: wgpu::Extent3d {
                width: atlas_w,
                height: atlas_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_w),
                rows_per_image: Some(atlas_h),
            },
            wgpu::Extent3d {
                width: atlas_w,
                height: atlas_h,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("time text atlas bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
    }

    fn set_overlay_text(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, text: &str) {
        let (canvas, w, h) = self.rasterize_text(text, UI_TIME_TEXT_FONT_SIZE);
        let w = w.max(1);
        let h = h.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("overlay text"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &canvas,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay text bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.overlay_texture = Some(texture);
        self.overlay_bind_group = Some(bind_group);
        self.overlay_width = w as f32;
        self.overlay_height = h as f32;
    }

    fn clear_overlay(&mut self) {
        self.overlay_texture = None;
        self.overlay_bind_group = None;
        self.overlay_width = 0.0;
        self.overlay_height = 0.0;
    }
}

fn compute_time_mark_interval(visible_duration: f32, target_marks: u32) -> f32 {
    let raw = visible_duration / target_marks as f32;
    let magnitude = 10f32.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let nice = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
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
    camera: Camera,
    background_color: Color,
    text_resolution_multiplier: f32,

    current_frame: u64,
    total_frames: u64,
    fps: u32,
    paused: bool,
    finished: bool,
    frame_rendered: bool,

    timeline_ops: Vec<TimelineOp>,
    original_animations: Vec<AnimOperation>,

    audio_player: Option<AudioPlayer>,
    waveform: Vec<f32>,
    sound_markers: Vec<SoundMarker>,

    ui_pipeline: RenderPipeline,
    ui_bind_group_layout: wgpu::BindGroupLayout,
    ui_uniform_buffer: wgpu::Buffer,
    ui_vertex_buffer: Option<wgpu::Buffer>,
    ui_index_buffer: Option<wgpu::Buffer>,
    ui_index_count: u32,
    ui_overlay_start: u32,

    selected_op_info: Option<String>,
    overlay_file: String,
    overlay_line: u32,
    overlay_anim_type: String,

    time_scale: f32,
    time_text_atlas: TimeTextAtlas,

    hovered_op: Option<usize>,
    hovered_sound: Option<usize>,
}

impl PreviewRenderer {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
        surface: Surface<'static>,
        surface_config: SurfaceConfiguration,
        camera: Camera,
        background_color: Color,
        animations: Vec<AnimOperation>,
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

        // Animator::new pops from the end, so we need to reverse
        let mut anims_for_animator = animations.clone();
        anims_for_animator.reverse();
        let animator = Animator::new(anims_for_animator, fps, camera, text_resolution_multiplier)?;

        let renderer = crate::renderer::Renderer::new(&device);

        let (ui_pipeline, ui_bind_group_layout, ui_uniform_buffer) =
            Self::create_ui_pipeline(&device)?;

        let time_text_atlas = TimeTextAtlas::new(&device);

        // Set up audio (starts paused; unpaused on first frame advance so
        // audio and video begin at the same wall‑clock time).
        let (audio_player, waveform, sound_markers) =
            Self::init_audio(&animations, surface_config.width, true);

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
            audio_player,
            waveform,
            sound_markers,
            ui_pipeline,
            ui_bind_group_layout,
            ui_uniform_buffer,
            ui_vertex_buffer: None,
            ui_index_buffer: None,
            ui_index_count: 0,
            ui_overlay_start: 0,
            selected_op_info: None,
            overlay_file: String::new(),
            overlay_line: 0,
            overlay_anim_type: String::new(),
            time_scale: 1.0,
            time_text_atlas,
            hovered_op: None,
            hovered_sound: None,
        })
    }

    fn init_audio(
        animations: &[AnimOperation],
        timeline_width: u32,
        initially_paused: bool,
    ) -> (Option<AudioPlayer>, Vec<f32>, Vec<SoundMarker>) {
        let sounds = collect_sounds_from_ops(animations);
        let sound_markers: Vec<SoundMarker> = sounds
            .iter()
            .map(|s| {
                let end = if s.duration > 0.0 {
                    Some(s.start_time + s.duration)
                } else {
                    None
                };
                let (file, line) = match &s.source_loc {
                    Some(loc) => (loc.file.clone(), loc.line),
                    None => (String::new(), 0),
                };
                let name = s.path.rsplit('/').next().unwrap_or(&s.path).to_string();
                SoundMarker {
                    start: s.start_time,
                    end,
                    file,
                    line,
                    name,
                }
            })
            .collect();
        if sounds.is_empty() {
            return (None, vec![], sound_markers);
        }
        let engine = AudioEngine::new(sounds);
        let pcm = match engine.mix() {
            Ok(pcm) => pcm,
            Err(e) => {
                log::error!("failed to mix audio for preview: {e}");
                return (None, vec![], sound_markers);
            }
        };
        if pcm.is_empty() {
            return (None, vec![], sound_markers);
        }
        let waveform = compute_waveform(&pcm, (timeline_width.max(1) as usize) * 50);
        let player = match AudioPlayer::new(pcm, crate::sfx::OUTPUT_SAMPLE_RATE) {
            Ok(p) => {
                p.set_paused(initially_paused);
                Some(p)
            }
            Err(e) => {
                log::error!("failed to create audio player: {e}");
                return (None, waveform, sound_markers);
            }
        };
        (player, waveform, sound_markers)
    }

    fn create_ui_pipeline(
        device: &wgpu::Device,
    ) -> Result<(RenderPipeline, wgpu::BindGroupLayout, wgpu::Buffer)> {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("ui shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("preview_ui.wgsl"))),
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ui uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<UIUniforms>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ui texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui pipeline layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let vertex_layout = VertexBufferLayout {
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
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ui pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[vertex_layout],
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

        Ok((pipeline, uniform_bind_group_layout, uniform_buffer))
    }

    fn total_duration_secs(&self) -> f32 {
        if self.total_frames > 0 {
            self.total_frames as f32 / self.fps as f32
        } else {
            1.0
        }
    }

    fn time_to_x(&self, time: f32, w: f32) -> f32 {
        let total_dur = self.total_duration_secs();
        if total_dur <= 0.0 {
            return 0.0;
        }
        if self.time_scale <= 1.0 {
            // Default zoom: full timeline fits the window width
            (time / total_dur) * w
        } else {
            // Zoomed in: center the view on the playhead
            let px_per_sec = (w / total_dur) * self.time_scale;
            let center_x = w / 2.0;
            let center_time = self.current_time();
            let dx = (time - center_time) * px_per_sec;
            center_x + dx
        }
    }

    fn x_to_time(&self, x: f32, w: f32) -> f32 {
        let total_dur = self.total_duration_secs();
        if total_dur <= 0.0 {
            return 0.0;
        }
        if self.time_scale <= 1.0 {
            // Default zoom: simple linear mapping
            ((x / w) * total_dur).clamp(0.0, total_dur)
        } else {
            // Zoomed in: center the view on the playhead
            let px_per_sec = (w / total_dur) * self.time_scale;
            let center_x = w / 2.0;
            let center_time = self.current_time();
            let dx = x - center_x;
            (center_time + dx / px_per_sec).clamp(0.0, total_dur)
        }
    }

    fn video_viewport(&self, w: u32, h: u32) -> (f32, f32, f32, f32) {
        let timeline_h = UI_TIMELINE_HEIGHT;
        let avail_h = (h as f32 - timeline_h).max(1.0);
        let scene_aspect = if self.camera.virtual_size.y > 0.0 {
            self.camera.virtual_size.x / self.camera.virtual_size.y
        } else {
            16.0 / 9.0
        };
        let avail_aspect = w as f32 / avail_h;

        let (vp_w, vp_h) = if avail_aspect > scene_aspect {
            (avail_h * scene_aspect, avail_h)
        } else {
            (w as f32, w as f32 / scene_aspect)
        };

        let vp_x = (w as f32 - vp_w) / 2.0;
        let vp_y = (avail_h - vp_h) / 2.0;
        (vp_x, vp_y, vp_w, vp_h)
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

        let current_time = self.current_time();
        let playhead_x = self.time_to_x(current_time, w);

        let mut vertices: Vec<UIVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Sentinel UV for solid color quads
        const NO_UV: [f32; 2] = [-1.0, -1.0];

        // Timeline background bar
        let bg_color = [0.05, 0.05, 0.1, 0.85];
        append_rect(
            &mut vertices,
            &mut indices,
            0.0,
            timeline_top,
            w,
            timeline_bottom,
            bg_color,
            NO_UV,
        );

        // Layout: time text at top, then ops, then waveform at bottom
        let text_y = timeline_top + 2.0;

        let marker_top = text_y + UI_TIME_TEXT_HEIGHT + 2.0;
        let marker_bot = marker_top + UI_OPERATION_MARKER_HEIGHT;
        let border_w = 3.0;

        let waveform_area_top = marker_bot + 2.0;
        let waveform_area_bot = timeline_bottom - 4.0;
        let waveform_center = (waveform_area_top + waveform_area_bot) / 2.0;
        let waveform_scale = (waveform_area_bot - waveform_area_top) / 2.0;

        // Audio waveform – only draw where SFX is playing
        if !self.waveform.is_empty() && !self.sound_markers.is_empty() {
            let wave_color = [0.2, 0.9, 0.7, 0.4];
            let audio_total = self
                .audio_player
                .as_ref()
                .map_or(0.0, |p| p.total_duration());
            if audio_total > 0.0 {
                let n = self.waveform.len();
                // For each sound marker range, draw one rect per visible pixel column
                // with the peak waveform value in that column's time span
                for (si, marker) in self.sound_markers.iter().enumerate() {
                    let is_hovered = self.hovered_sound == Some(si);
                    let end_t = marker.end.unwrap_or(marker.start + 1.0);
                    let sound_start = marker.start;
                    if end_t <= 0.0 {
                        continue;
                    }
                    let range_start_x = self.time_to_x(sound_start, w);
                    let range_end_x = self.time_to_x(end_t, w);
                    if range_end_x < 0.0 || range_start_x > w {
                        continue;
                    }
                    let visible_start = range_start_x.max(0.0);
                    let visible_end = range_end_x.min(w);
                    if visible_end <= visible_start {
                        continue;
                    }
                    let px_start = visible_start.ceil() as i32;
                    let px_end = visible_end.floor() as i32;
                    let wc = if is_hovered {
                        [0.2, 1.0, 0.8, 0.8]
                    } else {
                        wave_color
                    };
                    for px in px_start..=px_end {
                        let t0 = self.x_to_time(px as f32, w).max(sound_start);
                        let t1 = self.x_to_time((px + 1) as f32, w).min(end_t);
                        if t1 <= t0 {
                            continue;
                        }
                        let si0 = ((t0 / audio_total) * n as f32) as usize;
                        let si1 = ((t1 / audio_total) * n as f32) as usize;
                        if si1 <= si0 || si0 >= n {
                            continue;
                        }
                        let peak = {
                            let end = si1.min(n);
                            let mut p = 0.0f32;
                            for &s in &self.waveform[si0..end] {
                                if s > p {
                                    p = s;
                                }
                            }
                            p
                        };
                        let half_h = (peak * waveform_scale).max(1.0);
                        append_rect(
                            &mut vertices,
                            &mut indices,
                            px as f32,
                            waveform_center - half_h,
                            (px + 1) as f32,
                            waveform_center + half_h,
                            wc,
                            NO_UV,
                        );
                    }
                    // Sound start marker (blue) and end marker (red)
                    let sx = range_start_x;
                    if sx >= 0.0 && sx <= w {
                        let start_color = if is_hovered {
                            [0.5, 0.8, 1.0, 1.0]
                        } else {
                            [0.2, 0.5, 1.0, 0.9]
                        };
                        append_rect(
                            &mut vertices,
                            &mut indices,
                            sx - 1.0,
                            waveform_area_top,
                            sx + 1.0,
                            waveform_area_bot,
                            start_color,
                            NO_UV,
                        );
                    }
                    if marker.end.is_some() {
                        let ex = range_end_x;
                        if ex >= 0.0 && ex <= w {
                            let end_color = if is_hovered {
                                [1.0, 0.5, 0.5, 1.0]
                            } else {
                                [1.0, 0.2, 0.2, 0.9]
                            };
                            append_rect(
                                &mut vertices,
                                &mut indices,
                                ex - 1.0,
                                waveform_area_top,
                                ex + 1.0,
                                waveform_area_bot,
                                end_color,
                                NO_UV,
                            );
                        }
                    }
                }
            }
        }

        // Operation markers
        for (i, op) in self.timeline_ops.iter().enumerate() {
            let op_start_x = self.time_to_x(op.start_time, w);
            let op_end_x = self.time_to_x(op.end_time, w);
            if op_end_x < 0.0 || op_start_x > w {
                continue;
            }
            let mut color = op_color(op.kind);
            let border_color: [f32; 4];
            if self.hovered_op == Some(i) {
                border_color = [1.0, 1.0, 1.0, 1.0];
                color[0] = color[0] * 0.3 + 1.0 * 0.7;
                color[1] = color[1] * 0.3 + 1.0 * 0.7;
                color[2] = color[2] * 0.3 + 1.0 * 0.7;
            } else {
                border_color = [0.08, 0.08, 0.12, 1.0];
            }
            let visible_start = op_start_x.max(0.0);
            let visible_end = op_end_x.min(w);
            if visible_end <= visible_start {
                continue;
            }
            let visible_w = visible_end - visible_start;
            append_rect(
                &mut vertices,
                &mut indices,
                visible_start,
                marker_top,
                visible_start + visible_w,
                marker_bot,
                border_color,
                NO_UV,
            );
            let fill_x0 = visible_start + border_w;
            let fill_x1 = (visible_start + visible_w - border_w).max(fill_x0);
            let fill_y0 = marker_top + border_w;
            let fill_y1 = marker_bot - border_w;
            if fill_x1 > fill_x0 && fill_y1 > fill_y0 {
                append_rect(
                    &mut vertices,
                    &mut indices,
                    fill_x0,
                    fill_y0,
                    fill_x1,
                    fill_y1,
                    color,
                    NO_UV,
                );
            }
        }

        // Time tick marks + labels
        {
            let text_color = [1.0, 1.0, 1.0, 0.95];

            let visible_dur = self.x_to_time(w, w) - self.x_to_time(0.0, w);
            let interval = compute_time_mark_interval(visible_dur, 10);

            if interval > 0.0 {
                let first_mark = (self.x_to_time(0.0, w) / interval).ceil() * interval;
                let last_mark = self.x_to_time(w, w);

                // Use integer indexing for deterministic mark times (avoids FP drift in += loop)
                let n_marks = if interval > 0.0 {
                    ((last_mark - first_mark) / interval).floor() as i32 + 1
                } else {
                    0
                };

                let mut label_data: Vec<(f32, String, f32)> = Vec::new();
                for i in 0..n_marks {
                    let mark_time = first_mark + i as f32 * interval;
                    let mx = self.time_to_x(mark_time, w);
                    if mx >= 0.0 && mx <= w && mark_time >= 0.0 {
                        let label = if mark_time >= 60.0 {
                            let m = (mark_time / 60.0) as u32;
                            let s = mark_time % 60.0;
                            format!("{}m{:04.1}s", m, s)
                        } else {
                            format!("{:.1}s", mark_time)
                        };
                        label_data.push((mark_time, label, mx));
                    }
                }

                // Rebuild text atlas on explicit dirty (zoom/resize/reload) or when label set changes
                if self.time_text_atlas.dirty
                    || label_data.len() != self.time_text_atlas.labels.len()
                {
                    self.time_text_atlas
                        .rebuild(&self.device, &self.queue, &label_data);
                }

                // Update label positions
                let w_ = w;
                let x_positions: Vec<f32> = self
                    .time_text_atlas
                    .labels
                    .iter()
                    .map(|label| self.time_to_x(label.time, w_))
                    .collect();
                for (label, x) in self.time_text_atlas.labels.iter_mut().zip(x_positions) {
                    label.x_pixel = x;
                }

                // Draw textured quads for each label
                for label in &self.time_text_atlas.labels {
                    let lx = label.x_pixel;
                    if lx < 0.0 || lx > w {
                        continue;
                    }
                    let uv = label.uv_rect;
                    let label_w = (uv[2] - uv[0]) * self.time_text_atlas.atlas_width as f32;
                    let label_h = (uv[3] - uv[1]) * self.time_text_atlas.atlas_height as f32;

                    let x0 = lx - label_w / 2.0;
                    let x1 = x0 + label_w;
                    let y0 = text_y;
                    let y1 = y0 + label_h;

                    append_rect_textured(
                        &mut vertices,
                        &mut indices,
                        x0,
                        y0,
                        x1,
                        y1,
                        text_color,
                        [uv[0], uv[1]],
                        [uv[2], uv[3]],
                    );
                }
            }
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
            NO_UV,
        );

        // Top-right overlay for selected op info
        if self.time_text_atlas.overlay_bind_group.is_some() {
            let ow = self.time_text_atlas.overlay_width + 16.0;
            let oh = self.time_text_atlas.overlay_height + 8.0;
            let ox = w - ow - 8.0;
            let oy = 8.0;
            let bg = [0.05, 0.05, 0.1, 0.85];
            append_rect(
                &mut vertices,
                &mut indices,
                ox,
                oy,
                ox + ow,
                oy + oh,
                bg,
                NO_UV,
            );
            let tx = ox + 8.0;
            let ty = oy + 4.0;
            self.ui_overlay_start = indices.len() as u32;
            append_rect_textured(
                &mut vertices,
                &mut indices,
                tx,
                ty,
                tx + self.time_text_atlas.overlay_width,
                ty + self.time_text_atlas.overlay_height,
                [1.0, 1.0, 1.0, 0.95],
                [0.0, 0.0],
                [1.0, 1.0],
            );
        } else {
            self.ui_overlay_start = indices.len() as u32;
        }

        self.ui_index_count = indices.len() as u32;

        if vertices.is_empty() {
            self.ui_vertex_buffer = None;
            self.ui_index_buffer = None;
            return;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.ui_vertex_buffer = Some(vertex_buffer);
        self.ui_index_buffer = Some(index_buffer);
    }

    /// Advance the animator by one frame and render + present.
    /// Returns false when the animation is finished.
    pub fn advance_render_present(&mut self) -> Result<bool> {
        if self.finished {
            return Ok(false);
        }

        // Unpause audio on first frame advance so both start at the same
        // wall‑clock time (avoids audio building up a lead during startup).
        if !self.paused && self.current_frame == 0 && self.audio_player.is_some() {
            if let Some(ref player) = self.audio_player {
                player.set_paused(false);
            }
        }

        // Pre-compute viewport for video letterboxing
        let viewport = self.video_viewport(self.surface_config.width, self.surface_config.height);

        let have_scene = if !self.paused && self.current_frame < self.total_frames {
            // Advance the animator
            let frame_data = self.animator.animate_next_frame().with_context(|| {
                format!(
                    "while advancing animation frame {}/{}",
                    self.current_frame + 1,
                    self.total_frames,
                )
            })?;

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
                        CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
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

                    let mut encoder =
                        self.device
                            .create_command_encoder(&CommandEncoderDescriptor {
                                label: Some("Preview Frame Encoder"),
                            });

                    // Render scene with letterboxed viewport
                    let (vp_x, vp_y, vp_w, vp_h) = viewport;
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
                        rpass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);
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
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => {
                        return Ok(true);
                    }
                };
                let surface_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("Preview Frame Encoder"),
                    });

                let scene = crate::animator::Scene {
                    mesh_changed_this_frame: true,
                    camera: &self.animator.camera,
                    object_lookup: &self.animator.objects_lookup,
                    objects_sorted_by_z: &self.animator.objects,
                    vertices: &self.animator.vertices,
                    indices: &self.animator.indices,
                };

                // Render scene with letterboxed viewport
                let (vp_x, vp_y, vp_w, vp_h) = viewport;
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
                    rpass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);
                    if let Err(e) = draw_scene(
                        &mut rpass,
                        &self.pipelines,
                        &self.device,
                        &self.queue,
                        scene,
                        &mut self.renderer,
                    ) {
                        log::error!("Render scene in pause: {e:#}");
                    }
                }

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

    fn render_ui_pass(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let Some(ref vertex_buffer) = self.ui_vertex_buffer else {
            return;
        };
        let Some(ref index_buffer) = self.ui_index_buffer else {
            return;
        };
        if self.ui_index_count == 0 {
            return;
        }

        // Update uniforms
        let uniforms = UIUniforms {
            resolution: [
                self.surface_config.width as f32,
                self.surface_config.height as f32,
            ],
        };
        self.queue
            .write_buffer(&self.ui_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui uniform bind group"),
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
        rpass.set_bind_group(0, &uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Draw main UI (timeline, time text, ops, waveform, playhead, overlay background)
        if self.ui_overlay_start > 0 {
            if let Some(ref texture_bg) = self.time_text_atlas.bind_group {
                rpass.set_bind_group(1, texture_bg, &[]);
            }
            rpass.draw_indexed(0..self.ui_overlay_start, 0, 0..1);
        }

        // Draw overlay text quad with its own texture bind group
        if self.ui_overlay_start < self.ui_index_count {
            if let Some(ref overlay_bg) = self.time_text_atlas.overlay_bind_group {
                rpass.set_bind_group(1, overlay_bg, &[]);
            }
            rpass.draw_indexed(self.ui_overlay_start..self.ui_index_count, 0, 0..1);
        }
    }

    pub fn seek_to(&mut self, time_sec: f32) -> Result<()> {
        let frame = (time_sec * self.fps as f32).round() as u64;
        self.current_frame = frame.min(self.total_frames);
        self.finished = false;
        self.frame_rendered = false;

        // Animator::new pops from the end, so we need to reverse
        let mut anims_for_animator = self.original_animations.clone();
        anims_for_animator.reverse();
        let mut animator = Animator::new(
            anims_for_animator,
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

        if let Some(ref player) = self.audio_player {
            player.seek_to(self.current_time());
        }

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
        if let Some(ref player) = self.audio_player {
            player.set_paused(self.paused);
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        let was = self.paused;
        self.paused = paused;
        if was != paused {
            self.frame_rendered = false;
        }
        if let Some(ref player) = self.audio_player {
            player.set_paused(paused);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn needs_redraw(&self) -> bool {
        !self.frame_rendered
    }

    pub fn replay(&mut self) -> Result<()> {
        self.seek_to(0.0)
    }

    pub fn step_forward(&mut self) -> Result<bool> {
        self.paused = true;
        if let Some(ref player) = self.audio_player {
            player.set_paused(true);
        }
        if self.current_frame < self.total_frames {
            let frame_data = self
                .animator
                .animate_next_frame()
                .context("while stepping forward")?;
            match frame_data {
                Some(data) => {
                    self.current_frame += 1;
                    if self.current_frame >= self.total_frames {
                        self.finished = true;
                    }
                    if let Some(glyph_data) = data.glyph_update_data {
                        self.text_renderer
                            .update_glyphs_if_needed(glyph_data, &self.queue);
                    }
                }
                None => {
                    self.finished = true;
                    return Ok(false);
                }
            }
            self.frame_rendered = false;
            if let Some(ref player) = self.audio_player {
                player.seek_to(self.current_time());
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn step_backward(&mut self) -> Result<()> {
        self.paused = true;
        self.frame_rendered = false;
        if let Some(ref player) = self.audio_player {
            player.set_paused(true);
        }
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

        // Recompute waveform at high resolution for zoomable detail
        if let Some(ref player) = self.audio_player {
            self.waveform = compute_waveform(player.buffer(), (width.max(1) as usize) * 50);
        }

        // Force time text atlas rebuild
        self.time_text_atlas.dirty = true;
    }

    pub fn reload(&mut self, animations: Vec<AnimOperation>) -> Result<()> {
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

        // Animator::new pops from the end, so we need to reverse
        let mut anims_for_animator = self.original_animations.clone();
        anims_for_animator.reverse();
        self.animator = Animator::new(
            anims_for_animator,
            self.fps,
            self.camera,
            self.text_resolution_multiplier,
        )?;

        // Re-init audio (starts paused; unpaused on first frame advance)
        let (audio_player, waveform, sound_markers) =
            Self::init_audio(&self.original_animations, self.surface_config.width, true);
        self.audio_player = audio_player;
        self.waveform = waveform;
        self.sound_markers = sound_markers;

        self.time_text_atlas.dirty = true;

        Ok(())
    }

    pub fn timeline_ops(&self) -> &[TimelineOp] {
        &self.timeline_ops
    }

    pub fn animator(&self) -> &Animator {
        &self.animator
    }

    pub fn find_op_at_x(&self, x: f32) -> Option<&TimelineOp> {
        let w = self.surface_config.width as f32;
        if w <= 0.0 {
            return None;
        }
        let click_time = self.x_to_time(x, w);
        self.timeline_ops
            .iter()
            .rev()
            .find(|op| click_time >= op.start_time && click_time <= op.end_time)
    }

    pub fn find_sound_at_x(&self, x: f32) -> Option<(f32, Option<f32>, usize, &SoundMarker)> {
        let w = self.surface_config.width as f32;
        if w <= 0.0 {
            return None;
        }
        let click_time = self.x_to_time(x, w);
        for (i, marker) in self.sound_markers.iter().enumerate() {
            let end_time = marker.end.unwrap_or(marker.start);
            if click_time >= marker.start && click_time <= end_time {
                return Some((marker.start, marker.end, i, marker));
            }
        }
        None
    }

    pub fn window_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    pub fn set_selected_op(&mut self, op: Option<(&str, u32, &str)>) {
        if let Some((file, line, label)) = op {
            self.selected_op_info = Some(format!("{} @ {}:{}", label, file, line));
            self.overlay_file = file.to_string();
            self.overlay_line = line;
            self.overlay_anim_type = label.to_string();
            self.time_text_atlas.set_overlay_text(
                &self.device,
                &self.queue,
                &format!("{}:{} {}", file, line, label),
            );
        } else {
            self.selected_op_info = None;
            self.overlay_file.clear();
            self.overlay_line = 0;
            self.overlay_anim_type.clear();
            self.time_text_atlas.clear_overlay();
        }
        self.frame_rendered = false;
    }

    pub fn set_hovered_from_cursor(&mut self, cx: f32, cy: f32) {
        let h = self.surface_config.height as f32;
        let timeline_top = h - 80.0;
        let marker_bot = h - 34.0;
        let prev_op = self.hovered_op;
        let prev_sound = self.hovered_sound;
        if cy >= timeline_top && cy <= marker_bot {
            let idx = self.timeline_ops.iter().rposition(|op| {
                let t = self.x_to_time(cx, self.surface_config.width as f32);
                t >= op.start_time && t <= op.end_time
            });
            self.hovered_op = idx;
            self.hovered_sound = None;
        } else if cy >= marker_bot && cy <= h {
            let idx = self.sound_markers.iter().position(|m| {
                let t = self.x_to_time(cx, self.surface_config.width as f32);
                let end = m.end.unwrap_or(m.start);
                t >= m.start && t <= end
            });
            self.hovered_sound = idx;
            self.hovered_op = None;
        } else {
            self.hovered_op = None;
            self.hovered_sound = None;
        }
        if self.hovered_op != prev_op || self.hovered_sound != prev_sound {
            self.frame_rendered = false;
        }
    }

    pub fn zoom_in(&mut self) {
        self.time_scale = (self.time_scale * 1.3).min(50.0);
        self.time_text_atlas.dirty = true;
        self.frame_rendered = false;
    }

    pub fn zoom_out(&mut self) {
        if self.time_scale <= 0.2 {
            self.time_scale = 1.0;
        } else {
            self.time_scale = (self.time_scale / 1.3).max(0.2);
        }
        self.time_text_atlas.dirty = true;
        self.frame_rendered = false;
    }

    pub fn zoom_reset(&mut self) {
        self.time_scale = 1.0;
        self.time_text_atlas.dirty = true;
        self.frame_rendered = false;
    }

    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Convert a screen x position to a time value, accounting for zoom.
    pub fn screen_x_to_time(&self, x: f32) -> f32 {
        let w = self.surface_config.width as f32;
        if w <= 0.0 {
            return 0.0;
        }
        self.x_to_time(x, w)
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
    uv: [f32; 2],
) {
    let base = vertices.len() as u32;
    vertices.push(UIVertex {
        position: [x0, y0],
        color,
        uv,
    });
    vertices.push(UIVertex {
        position: [x1, y0],
        color,
        uv,
    });
    vertices.push(UIVertex {
        position: [x1, y1],
        color,
        uv,
    });
    vertices.push(UIVertex {
        position: [x0, y1],
        color,
        uv,
    });
    indices.push(base);
    indices.push(base + 1);
    indices.push(base + 2);
    indices.push(base);
    indices.push(base + 2);
    indices.push(base + 3);
}

fn append_rect_textured(
    vertices: &mut Vec<UIVertex>,
    indices: &mut Vec<u32>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: [f32; 4],
    uv_min: [f32; 2],
    uv_max: [f32; 2],
) {
    let base = vertices.len() as u32;
    vertices.push(UIVertex {
        position: [x0, y0],
        color,
        uv: [uv_min[0], uv_min[1]],
    });
    vertices.push(UIVertex {
        position: [x1, y0],
        color,
        uv: [uv_max[0], uv_min[1]],
    });
    vertices.push(UIVertex {
        position: [x1, y1],
        color,
        uv: [uv_max[0], uv_max[1]],
    });
    vertices.push(UIVertex {
        position: [x0, y1],
        color,
        uv: [uv_min[0], uv_max[1]],
    });
    indices.push(base);
    indices.push(base + 1);
    indices.push(base + 2);
    indices.push(base);
    indices.push(base + 2);
    indices.push(base + 3);
}
