// Phase 0: Minimal terminal — PTY + GPU text rendering + basic input
// ponytail: no tab/split/profile/UI yet — single terminal, no scrollback, raw PTY

use anyhow::Result;
use libghostty_vt::{
    Terminal, TerminalOptions, RenderState,
    key::Encoder as KeyEncoder,
    key::Event as KeyEventGhostty,
    key::Action as KeyAction,
    key::Key as GhosttyKey,
    mouse::Encoder as MouseEncoder,
};
use pollster;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key as WinitKey, NamedKey},
    window::{Window, WindowAttributes},
};
use winit::dpi::PhysicalSize;
use egui_winit::State as EguiWinitState;
use egui_wgpu::Renderer as EguiWgpuRenderer;
use egui::Context;

struct App<'alloc> {
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    egui_state: Option<EguiWinitState>,
    egui_renderer: Option<EguiWgpuRenderer>,
    terminal: Option<Terminal<'alloc, 'alloc>>,
    render_state: Option<RenderState<'alloc>>,
    key_encoder: KeyEncoder<'alloc>,
    mouse_encoder: MouseEncoder<'alloc>,
    pty: Option<PtyHandle>,
    size: PhysicalSize<u32>,
}

struct PtyHandle {
    master: Box<dyn portable_pty::MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyHandle {
    fn spawn(cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut builder = CommandBuilder::new("cmd");
        #[cfg(not(windows))]
        { builder = CommandBuilder::new("bash"); }
        
        builder.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(builder)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        eprintln!("[PTY] spawn {}x{}", cols, rows);
        Ok(Self { master: pair.master, reader, writer, _child: child })
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        let _ = self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
        eprintln!("[PTY] resize {}x{}", cols, rows);
    }

    fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
    }

    fn try_read(&mut self) -> Option<Vec<u8>> {
        let mut buf = [0u8; 4096];
        match self.reader.read(&mut buf) {
            Ok(0) => None,
            Ok(n) => Some(buf[..n].to_vec()),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(e) => {
                eprintln!("[PTY] read error: {e}");
                None
            }
        }
    }
}

fn winit_key_to_ghostty(key: &WinitKey) -> Option<GhosttyKey> {
    match key {
        WinitKey::Named(named) => match named {
            NamedKey::Enter => Some(GhosttyKey::Enter),
            NamedKey::Tab => Some(GhosttyKey::Tab),
            NamedKey::Backspace => Some(GhosttyKey::Backspace),
            NamedKey::Escape => Some(GhosttyKey::Escape),
            NamedKey::Space => Some(GhosttyKey::Space),
            NamedKey::ArrowUp => Some(GhosttyKey::ArrowUp),
            NamedKey::ArrowDown => Some(GhosttyKey::ArrowDown),
            NamedKey::ArrowLeft => Some(GhosttyKey::ArrowLeft),
            NamedKey::ArrowRight => Some(GhosttyKey::ArrowRight),
            NamedKey::Home => Some(GhosttyKey::Home),
            NamedKey::End => Some(GhosttyKey::End),
            NamedKey::PageUp => Some(GhosttyKey::PageUp),
            NamedKey::PageDown => Some(GhosttyKey::PageDown),
            NamedKey::Delete => Some(GhosttyKey::Delete),
            NamedKey::Insert => Some(GhosttyKey::Insert),
            NamedKey::F1 => Some(GhosttyKey::F1),
            NamedKey::F2 => Some(GhosttyKey::F2),
            NamedKey::F3 => Some(GhosttyKey::F3),
            NamedKey::F4 => Some(GhosttyKey::F4),
            NamedKey::F5 => Some(GhosttyKey::F5),
            NamedKey::F6 => Some(GhosttyKey::F6),
            NamedKey::F7 => Some(GhosttyKey::F7),
            NamedKey::F8 => Some(GhosttyKey::F8),
            NamedKey::F9 => Some(GhosttyKey::F9),
            NamedKey::F10 => Some(GhosttyKey::F10),
            NamedKey::F11 => Some(GhosttyKey::F11),
            NamedKey::F12 => Some(GhosttyKey::F12),
            _ => None,
        },
        WinitKey::Character(c) => {
            let s = c.as_str();
            if s.len() == 1 {
                let ch = s.chars().next().unwrap();
                // Map character to GhosttyKey using match
                match ch {
                    'a' => Some(GhosttyKey::A),
                    'b' => Some(GhosttyKey::B),
                    'c' => Some(GhosttyKey::C),
                    'd' => Some(GhosttyKey::D),
                    'e' => Some(GhosttyKey::E),
                    'f' => Some(GhosttyKey::F),
                    'g' => Some(GhosttyKey::G),
                    'h' => Some(GhosttyKey::H),
                    'i' => Some(GhosttyKey::I),
                    'j' => Some(GhosttyKey::J),
                    'k' => Some(GhosttyKey::K),
                    'l' => Some(GhosttyKey::L),
                    'm' => Some(GhosttyKey::M),
                    'n' => Some(GhosttyKey::N),
                    'o' => Some(GhosttyKey::O),
                    'p' => Some(GhosttyKey::P),
                    'q' => Some(GhosttyKey::Q),
                    'r' => Some(GhosttyKey::R),
                    's' => Some(GhosttyKey::S),
                    't' => Some(GhosttyKey::T),
                    'u' => Some(GhosttyKey::U),
                    'v' => Some(GhosttyKey::V),
                    'w' => Some(GhosttyKey::W),
                    'x' => Some(GhosttyKey::X),
                    'y' => Some(GhosttyKey::Y),
                    'z' => Some(GhosttyKey::Z),
                    'A' => Some(GhosttyKey::A),
                    'B' => Some(GhosttyKey::B),
                    'C' => Some(GhosttyKey::C),
                    'D' => Some(GhosttyKey::D),
                    'E' => Some(GhosttyKey::E),
                    'F' => Some(GhosttyKey::F),
                    'G' => Some(GhosttyKey::G),
                    'H' => Some(GhosttyKey::H),
                    'I' => Some(GhosttyKey::I),
                    'J' => Some(GhosttyKey::J),
                    'K' => Some(GhosttyKey::K),
                    'L' => Some(GhosttyKey::L),
                    'M' => Some(GhosttyKey::M),
                    'N' => Some(GhosttyKey::N),
                    'O' => Some(GhosttyKey::O),
                    'P' => Some(GhosttyKey::P),
                    'Q' => Some(GhosttyKey::Q),
                    'R' => Some(GhosttyKey::R),
                    'S' => Some(GhosttyKey::S),
                    'T' => Some(GhosttyKey::T),
                    'U' => Some(GhosttyKey::U),
                    'V' => Some(GhosttyKey::V),
                    'W' => Some(GhosttyKey::W),
                    'X' => Some(GhosttyKey::X),
                    'Y' => Some(GhosttyKey::Y),
                    'Z' => Some(GhosttyKey::Z),
                    '0' => Some(GhosttyKey::Digit0),
                    '1' => Some(GhosttyKey::Digit1),
                    '2' => Some(GhosttyKey::Digit2),
                    '3' => Some(GhosttyKey::Digit3),
                    '4' => Some(GhosttyKey::Digit4),
                    '5' => Some(GhosttyKey::Digit5),
                    '6' => Some(GhosttyKey::Digit6),
                    '7' => Some(GhosttyKey::Digit7),
                    '8' => Some(GhosttyKey::Digit8),
                    '9' => Some(GhosttyKey::Digit9),
                    ' ' => Some(GhosttyKey::Space),
                    '\t' => Some(GhosttyKey::Tab),
                    '\n' => Some(GhosttyKey::Enter),
                    '\r' => Some(GhosttyKey::Enter),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

impl<'alloc> App<'alloc> {
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
            key_encoder: KeyEncoder::new().unwrap(),
            mouse_encoder: MouseEncoder::new().unwrap(),
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
        let egui_state = EguiWinitState::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, None);
        let egui_renderer = EguiWgpuRenderer::new(&device, format, None, 1, true);

        // libghostty-vt terminal
        let terminal = Terminal::new(TerminalOptions {
            cols: (self.size.width / 10) as u16,
            rows: (self.size.height / 20) as u16,
            max_scrollback: 0, // Phase 0: no scrollback
        })?;
        let render_state = RenderState::new()?;

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
            let _ = terminal.resize(cols, rows, 10, 20);
            pty.resize(cols, rows);
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn handle_key(&mut self, event: &KeyEvent) {
        if let Some(terminal) = &mut self.terminal {
            if let Some(ghostty_key) = winit_key_to_ghostty(&event.logical_key) {
                let mut key_event = KeyEventGhostty::new().unwrap();
                key_event.set_key(ghostty_key);
                key_event.set_action(match event.state {
                    ElementState::Pressed => KeyAction::Press,
                    ElementState::Released => KeyAction::Release,
                });
                
                let mut buf = Vec::new();
                if self.key_encoder.encode_to_vec(&key_event, &mut buf).is_ok() {
                    if !buf.is_empty() {
                        if let Some(pty) = &mut self.pty {
                            pty.write(&buf);
                        }
                    }
                }
            }
        }
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
                terminal.vt_write(&data);
            }
        }

        // Update render state from terminal
        let _snapshot = render_state.update(terminal)?;

        // wgpu frame
        let frame = surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ponytail: glyph atlas + grid pipeline goes here
        // For Phase 0, just clear and draw egui placeholder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        
        // Take egui input BEFORE getting context
        let raw_input = egui_state.take_egui_input(&window);
        let egui_ctx = egui_state.egui_ctx();
        
        let full_output = egui_ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Schorl Terminal — Phase 0");
                ui.label(format!("Size: {}x{}", self.size.width, self.size.height));
                if let Some(term) = &self.terminal {
                    ui.label(format!("Grid: {}x{}", term.cols().unwrap_or(0), term.rows().unwrap_or(0)));
                }
            });
        });
        
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            // egui_wgpu requires 'static lifetime on render pass
            let mut rpass = rpass.forget_lifetime();
            
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.size.width, self.size.height],
                pixels_per_point: window.scale_factor() as f32,
            };
            egui_renderer.render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
        window.request_redraw();
        Ok(())
    }
}

impl<'alloc> ApplicationHandler for App<'alloc> {
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
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app: App<'static> = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}