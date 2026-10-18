# Schorl Terminal

A fast, GPU-accelerated terminal emulator for Windows, built with Rust.

**Stack:** `libghostty-vt` (VT emulation) + `wgpu` (GPU rendering) + `egui` (UI)

## Status: Phase 0 — Minimal Foundation

Currently: single terminal window, black background, egui placeholder UI. No PTY, no glyph rendering, no tabs/splits.

See `DESIGN.md` for the full phased roadmap.

## Requirements

- Rust 1.85+
- Zig 0.13+ (for building `libghostty-vt-sys` from source)
- Windows 10/11, Linux, or macOS

## Build & Run

```bash
cargo build    # First build: 2-5 min (fetches/builds Ghostty via Zig)
cargo run      # Opens window
```

## Project Structure

```
schorl-terminal/
├── Cargo.toml          # Dependencies
├── DESIGN.md           # Phased implementation plan
├── AGENTS.md           # Agent guide
└── src/
    └── main.rs         # Entire app (Phase 0: single file)
```

## Architecture (Phase 0)

```
App (ApplicationHandler)
├── winit window + event loop
├── wgpu device/queue/surface
├── egui_winit::State + egui_wgpu::Renderer
├── libghostty_vt::Terminal + RenderState
├── KeyEncoder / MouseEncoder → PTY
└── PtyHandle (stub)
```

## Next Steps (Phase 1)

1. Real PTY via `portable-pty`
2. Glyph atlas (fontdue + wgpu texture)
3. Grid render pipeline (instanced quads)
4. Scrollback + resize reflow

## License

MIT