// Phase 0: Minimal terminal — PTY + GPU text rendering + basic input
// ponytail: no tab/split/profile/UI yet — single terminal, no scrollback, raw PTY

use anyhow::Result;
use libghostty_vt::{Terminal, TerminalOptions, RenderState, KeyEncoder, MouseEncoder};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};
use winit::dpi::PhysicalSize;
use egui_winit::State as EguiWinitState;
use egui_wgpu::Renderer as EguiWgpuRenderer;
use egui::Context;

struct App {
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    egui_state: Option<EguiWinitState>,
    egui_renderer: Option<EguiWgpuRenderer>,
    terminal: Option<Terminal>,
    render_state: Option<RenderState>,
    key_encoder: KeyEncoder,
    mouse_encoder: MouseEncoder,
    pty: Option<PtyHandle>,
    size: PhysicalSize<u32>,
}

struct PtyHandle {
    // Placeholder — real impl uses ConPTY (Windows) or forkpty (Unix)
    _inner: (),
}

impl PtyHandle {
    fn spawn(cols: u16, rows: u16) -> Result<Self> {
        // ponytail: stub — replace with portable-pty or platform-specific
        eprintln!("[PTY] spawn {}x{} (stub)", cols, rows);
        Ok(Self { _inner: () })
    }
    fn resize(&mut self, cols: u16, rows: u16) {
        eprintln!("[PTY] resize {}x{} (stub)", cols, rows);
    }
    fn write(&mut self, data: &[u8]) {
        // ponytail: write to PTY
    }
    fn try_read(&mut self) -> Option<Vec<u8>> {
        // ponytail: read from PTY
        None
    }
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,
            egui_state: None,
            egui_renderer: None,
            terminal: None,
            render_state: None,
            key_encoder: KeyEncoder::new(),
            mouse_encoder: MouseEncoder::new(),
            pty: None,
            size: PhysicalSize::new(800, 600),
        }
    }

    async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        // Window
        let attrs = WindowAttributes::default()
            .with_title("Schorl Terminal")
            .with_inner_size(self.size);
        let window = Arc::new(event_loop.create_window(attrs)?);
        self.size = window.inner_size();
        self.window = Some(window.clone());

        // wgpu
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| anyhow::anyhow!("No adapter"))?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("terminal"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: self.size.width,
            height: self.size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // egui
        let egui_ctx = Context::default();
        let egui_state = EguiWinitState::new(egui_ctx, egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None);
        let egui_renderer = EguiWgpuRenderer::new(&device, format, None, 1);

        // libghostty-vt terminal
        let mut terminal = Terminal::new(TerminalOptions {
            cols: (self.size.width / 10) as u16,
            rows: (self.size.height / 20) as u16,
            max_scrollback: 0, // Phase 0: no scrollback
        })?;
        let render_state = RenderState::new();

        // PTY
        let pty = PtyHandle::spawn(
            (self.size.width / 10) as u16,
            (self.size.height / 20) as u16,
        )?;

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        self.terminal = Some(terminal);
        self.render_state = Some(render_state);
        self.pty = Some(pty);

        window.request_redraw();
        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        if let (Some(surface), Some(device), Some(config)) = (&self.surface, &self.device, &mut self.surface_config) {
            config.width = new_size.width.max(1);
            config.height = new_size.height.max(1);
            surface.configure(device, config);
        }
        if let (Some(terminal), Some(pty)) = (&mut self.terminal, &mut self.pty) {
            let cols = (new_size.width / 10).max(1) as u16;
            let rows = (new_size.height / 20).max(1) as u16;
            let _ = terminal.resize(cols, rows);
            pty.resize(cols, rows);
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn handle_key(&mut self, event: &KeyEvent) {
        if let Some(terminal) = &mut self.terminal {
            let encoded = self.key_encoder.encode_key(event);
            if !encoded.is_empty() {
                if let Some(pty) = &mut self.pty {
                    pty.write(&encoded);
                }
            }
        }
    }

    fn handle_mouse(&mut self, event: &WindowEvent) {
        // ponytail: minimal mouse — extend for selection, wheel, etc.
    }

    fn render(&mut self) -> Result<()> {
        let Some(window) = &self.window else { return Ok(()) };
        let Some(surface) = &self.surface else { return Ok(()) };
        let Some(device) = &self.device else { return Ok(()) };
        let Some(queue) = &self.queue else { return Ok(()) };
        let Some(terminal) = &mut self.terminal else { return Ok(()) };
        let Some(render_state) = &mut self.render_state else { return Ok(()) };
        let Some(egui_state) = &mut self.egui_state else { return Ok(()) };
        let Some(egui_renderer) = &mut self.egui_renderer else { return Ok(()) };

        // Read PTY output
        if let Some(pty) = &mut self.pty {
            while let Some(data) = pty.try_read() {
                terminal.feed(&data);
            }
        }

        // Render terminal to RenderState
        terminal.render(render_state);

        // wgpu frame
        let frame = surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ponytail: glyph atlas + grid pipeline goes here
        // For Phase 0, just clear and draw egui placeholder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // egui UI (placeholder)
            let egui_ctx = egui_state.egui_ctx();
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.label("Schorl Terminal — Phase 0");
                ui.label(format!("Size: {}x{}", self.size.width, self.size.height));
                if let Some(term) = &self.terminal {
                    ui.label(format!("Grid: {}x{}", term.cols(), term.rows()));
                }
            });

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.size.width, self.size.height],
                pixels_per_point: window.scale_factor() as f32,
            };
            egui_renderer.render(&mut rpass, egui_ctx, screen_descriptor, &egui_state);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
        window.request_redraw();
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(e) = pollster::block_on(self.init(event_loop)) {
                eprintln!("Init failed: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        let Some(window) = &self.window else { return };
        let Some(egui_state) = &mut self.egui_state else { return };

        let consumed = egui_state.on_window_event(window, &event).consumed;
        if consumed { return; }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    eprintln!("Render error: {e}");
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.handle_key(&event);
                }
            }
            WindowEvent::MouseInput { state, button, .. } |
            WindowEvent::CursorMoved { .. } |
            WindowEvent::MouseWheel { .. } => {
                self.handle_mouse(&event);
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
