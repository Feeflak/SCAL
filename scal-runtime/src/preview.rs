use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use notify::{EventKind, RecursiveMode, Watcher};
use scal::preview::PreviewRenderer;
use scal::projection::Camera;
use scal::types::Color;
use tokio::sync::watch;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use winit::platform::wayland::EventLoopBuilderExtWayland;

use crate::Config;

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
        animations: scal::convert_anim_ops(
            project.timeline,
            &project.scene_settings.default_theme,
        )?,
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
    animations: Vec<scal::anim_op::AnimOP>,
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
                if let Some(ref window) = self.window {
                    window.request_redraw();
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
                if seek_on_timeline(preview, self.window_size, self.cursor_pos) {
                    if let Some(ref window) = self.window {
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
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn seek_on_timeline(preview: &mut PreviewRenderer, window_size: (u32, u32), cursor_pos: (f64, f64)) -> bool {
    let (ww, wh) = window_size;
    if ww > 0 && wh > 0 {
        let (cx, cy) = cursor_pos;
        let timeline_top = wh as f64 - 48.0;
        if cy >= timeline_top && cy <= wh as f64 {
            let click_ratio = (cx / ww as f64).clamp(0.0, 1.0);
            let total_dur = preview.total_duration();
            let seek_time = click_ratio as f32 * total_dur;
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
    let shift = modifiers
        .as_ref()
        .map_or(false, |m| m.state().intersects(winit::keyboard::ModifiersState::SHIFT));
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
        Key::Character(c) if (c == "l" || c == "L") && shift => {
            seek_by_seconds(preview, 1.0)
        }
        Key::Character(c) if c == "l" || c == "L" => {
            if let Err(e) = preview.step_forward() {
                log::error!("Step forward error: {e}");
            }
        }
        Key::Character(c) if (c == "h" || c == "H") && shift => {
            seek_by_seconds(preview, -1.0)
        }
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

    let render_animations = scal::convert_anim_ops(project.timeline, default_theme)?;
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
