# Schorl Terminal — System Design & Implementation Plan

**Stack:** Rust + `libghostty-vt` (VT emulation) + `wgpu` (GPU rendering) + `egui` (UI)
**Target:** Windows 10/11, Linux, macOS
**Scope:** Full Windows Terminal parity (tabs, splits, profiles, settings, command palette)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Application                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Tab/Split  │  │  Profile    │  │  Settings/Commands  │  │
│  │  Manager    │  │  Manager    │  │  Palette            │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                    │             │
│         └────────────────┼────────────────────┘             │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Terminal Host                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │   │
│  │  │ Terminal A   │  │ Terminal B   │  │ Terminal N│  │   │
│  │  │ (libghostty) │  │ (libghostty) │  │ (libghostty)│   │
│  │  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  │   │
│  │         │                 │                 │          │   │
│  │         └─────────────────┼─────────────────┘          │   │
│  │                           ▼                            │   │
│  │              ┌───────────────────────┐                 │   │
│  │              │   Renderer (wgpu)     │                 │   │
│  │              │   - Atlas cache       │                 │   │
│  │              │   - Glyph pipeline    │                 │   │
│  │              │   - Grid renderer     │                 │   │
│  │              └───────────┬───────────┘                 │   │
│  └──────────────────────────┼──────────────────────────────┘   │
│                             ▼                                  │
│              ┌───────────────────────┐                         │
│              │   egui + winit        │                         │
│              │   - Window/Input      │                         │
│              │   - UI (tabs, menus)  │                         │
│              └───────────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 0 — Foundation (Week 1)

**Goal:** Minimal working terminal window — PTY + GPU text rendering + basic input

### Tasks
- [ ] Cargo workspace with `libghostty-vt`, `wgpu`, `winit`, `egui`, `egui-wgpu`, `egui-winit`
- [ ] Build `libghostty-vt` from source (Zig dependency)
- [ ] Create `TerminalHost` — owns `Terminal` + `RenderState` + wgpu renderer
- [ ] Implement glyph atlas (font texture) + basic grid shader
- [ ] Wire winit events → `KeyEncoder`/`MouseEncoder` → `Terminal`
- [ ] Render loop: `terminal.render()` → `RenderState` → wgpu draw calls
- [ ] Spawn PTY (ConPTY on Windows, forkpty on Unix) → pipe to `Terminal`

**Vertical Slice:** Single terminal, 80x24, raw PTY, no scrollback, no UI

---

## Phase 1 — Terminal Core (Week 2)

**Goal:** Production-grade terminal emulation

### Tasks
- [ ] Scrollback buffer (configurable, 10k+ lines)
- [ ] Dynamic resize (reflow + PTY resize)
- [ ] Unicode / grapheme cluster rendering
- [ ] True color (24-bit) + indexed color palette
- [ ] Cursor styles (block, bar, underline) + blink
- [ ] Selection (mouse + keyboard) + copy/paste
- [ ] OSC 52 clipboard sync
- [ ] Kitty graphics protocol (images in terminal)
- [ ] Sixel / ReGIS (optional, libghostty-vt support)
- [ ] Hyperlinks (OSC 8) + click handling

**Vertical Slice:** Full VT parity with Ghostty, GPU-rendered, copy/paste works

---

## Phase 2 — Multi-Terminal Host (Week 3)

**Goal:** Multiple terminals in one process, shared renderer

### Tasks
- [ ] `TerminalHost` manages `Vec<TerminalInstance>` 
- [ ] Shared glyph atlas across terminals (memory efficiency)
- [ ] Per-terminal render state + dirty tracking
- [ ] Focus management (input routing)
- [ ] PTY lifecycle (spawn, exit, restart, signal)
- [ ] Shell auto-detection (PowerShell, cmd, pwsh, bash, zsh, fish)

**Vertical Slice:** Multiple terminals, switchable, shared GPU resources

---

## Phase 3 — Tab & Split UI (Week 4-5)

**Goal:** Windows Terminal-style tabs + splits (egui-based)

### Tasks
- [ ] **Tab bar** (top): new tab, close, drag-reorder, rename, context menu
- [ ] **Split panes**: horizontal/vertical, resize drag handle, focus indicator
- [ ] **Tab context menu**: duplicate, close others, close right, move to new window
- [ ] **Keyboard shortcuts**: Ctrl+Shift+T, Ctrl+Shift+W, Ctrl+Shift+D, Alt+Arrows
- [ ] **Tab state persistence**: title, cwd, shell, split layout
- [ ] **Drag & drop tabs** between windows (egui DnD + winit multi-window)

**Vertical Slice:** Full tab/split UX, keyboard-driven, persists layout

---

## Phase 4 — Profiles & Settings (Week 5-6)

**Goal:** Windows Terminal-style profile system

### Tasks
- [ ] **Profile schema** (TOML/JSON):
  - Shell command + args + env
  - Starting directory
  - Font (face, size, ligatures, fallback)
  - Color scheme (name + overrides)
  - Cursor style, blink rate
  - Bell (audio/visual)
  - Tab title template
  - Key bindings (per-profile override)
- [ ] **Profile manager**: list, create, edit, delete, set default
- [ ] **Settings UI** (egui): profiles, appearance, keyboard, advanced
- [ ] **Color scheme editor**: import .itermcolors, .terminal, JSON
- [ ] **Font picker** (system fonts + fallback chain)
- [ ] **Config file**: `$APPDATA/schorl-terminal/settings.toml` (Windows)

**Vertical Slice:** Persistent profiles, settings UI, scheme/font management

---

## Phase 5 — Command Palette & Actions (Week 6)

**Goal:** Discoverable, searchable command system

### Tasks
- [ ] **Action registry**: typed commands with metadata (name, description, shortcut)
- [ ] **Command palette** (Ctrl+Shift+P): fuzzy search, recent, categories
- [ ] **Built-in actions**:
  - New tab/split/window (with profile selector)
  - Close tab/pane/window
  - Copy/paste/selection
  - Find (search scrollback)
  - Toggle fullscreen, zoom
  - Reload config
  - Open settings
  - Switch profile
  - Rename tab
- [ ] **Key binding editor** in settings (record chords, conflicts)

**Vertical Slice:** Full command palette, every action discoverable, rebindable

---

## Phase 6 — Advanced Features (Week 7-8)

**Goal:** Power-user features

### Tasks
- [ ] **Quake mode** (dropdown terminal, global hotkey)
- [ ] **Session restore**: save/restore tabs, splits, cwd, scrollback
- [ ] **Search UI**: regex, case-sensitive, highlight all, scrollbar marks
- [ ] **Background image/blur/opacity** (wgpu shader)
- [ ] **Terminal shaders** (CRT, scanlines, custom WGSL)
- [ ] **Mouse reporting modes** (SGR, urect, etc.)
- [ ] **Bracketed paste** + paste protection
- [ ] **Focus follows mouse** option

---

## Phase 7 — Polish & Packaging (Week 8-9)

**Goal:** Distributable app

### Tasks
- [ ] **MSIX/Store packaging** (Windows)
- [ ] **Auto-update** (GitHub releases + background check)
- [ ] **Installer** (WiX or cargo-wix)
- [ ] **Code signing** (cert + timestamp)
- [ ] **Telemetry opt-in** (crash reports, usage)
- [ ] **Accessibility** (UIA, screen readers)
- [ ] **Performance**: frame time < 8ms, startup < 500ms
- [ ] **CI/CD**: GitHub Actions (build, test, sign, release)

---

## Data Structures (Core Types)

```rust
// Terminal instance (one per pane)
struct TerminalInstance {
    id: Uuid,
    terminal: Terminal,           // libghostty-vt
    render_state: RenderState,    // libghostty-vt
    pty: PtyHandle,               // platform PTY
    profile: ProfileId,
    title: String,
    cwd: PathBuf,
    dirty: bool,                  // needs re-render
}

// Tab contains split tree
enum SplitNode {
    Leaf(TerminalInstance),
    Horizontal(Vec<SplitNode>, Vec<f32>), // children + ratios
    Vertical(Vec<SplitNode>, Vec<f32>),
}

struct Tab {
    id: Uuid,
    root: SplitNode,
    active_leaf: Uuid,
    title: String,
}

// Profile (serializable)
struct Profile {
    id: Uuid,
    name: String,
    shell: ShellSpec,
    font: FontSpec,
    colors: ColorScheme,
    cursor: CursorSpec,
    keybindings: KeyMap,
    // ... more
}
```

---

## Renderer Design (wgpu)

**Glyph Atlas:**
- Single `Texture2D` (2048x2048, RGBA8)
- Pack glyphs on-demand (stb_truetype / rusttype / fontdue)
- LRU eviction when full
- Shared across all terminals

**Pipelines:**
1. **Grid pipeline** — instanced quads, vertex = cell position, instance = glyph ID + color
2. **Cursor pipeline** — single quad, animated blink
3. **Selection pipeline** — semi-transparent overlay
4. **Background pipeline** — image/blur/shader

**Render Pass:**
```rust
fn render(terminal: &TerminalInstance, encoder: &mut CommandEncoder) {
    let render_state = terminal.terminal.render(&terminal.render_state);
    let cells = render_state.grid.cells(); // RowIterator + CellIterator
    
    // Update glyph atlas with new glyphs
    // Write instance buffer (position, glyph_id, fg, bg, attrs)
    // Draw grid pipeline
    // Draw cursor + selection overlays
}
```

---

## Input Flow

```
winit Event
    │
    ▼
egui-winit (consumes UI events: tab clicks, menu, etc.)
    │
    ▼ (if not consumed by UI)
TerminalHost::handle_event(event)
    │
    ├── KeyEvent → KeyEncoder.encode_key() → Terminal.feed()
    ├── MouseEvent → MouseEncoder.encode() → Terminal.feed()
    ├── Resize → Terminal.resize() + PTY resize
    └── Focus → Terminal.set_focus()
```

---

## Dependencies & Versions (Pinned)

```toml
[dependencies]
libghostty-vt = "0.2"        # VT core (requires Zig 0.13+ to build)
wgpu = "0.20"                # GPU abstraction
winit = "0.30"               # Window/input
egui = "0.29"                # Immediate UI
egui-wgpu = "0.29"           # egui wgpu backend
egui-winit = "0.29"          # egui winit integration
bytemuck = "1"               # Pod/Zeroable for GPU buffers
anyhow = "1"                 # Errors
serde = { version = "1", features = ["derive"] }
toml = "0.8"                 # Config
uuid = { version = "1", features = ["v4", "serde"] }
directories = "5"            # Config dirs
```

**Build-time:** Zig 0.13+ (for `libghostty-vt-sys`)

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `libghostty-vt` API unstable | High | High | Pin version, vendor if needed |
| wgpu/egui version drift | Medium | Medium | Lockfile, test on update |
| Windows ConPTY quirks | High | Medium | Test early, fallback to winpty |
| Font rendering quality | Medium | High | Use fontdue + subpixel AA |
| Multi-window DnD (egui) | Medium | Medium | Prototype early |
| MSIX packaging complexity | Low | High | Defer to Phase 7 |

---

## Milestone Gates

| Phase | Gate | Criteria |
|-------|------|----------|
| 0 | M0 | `cargo run` → window with working shell |
| 1 | M1 | `vim`, `ls --color`, `htop` render correctly |
| 2 | M2 | 10 terminals, switch instantly, no leaks |
| 3 | M3 | Tabs + splits + drag-reorder + keyboard nav |
| 4 | M4 | Settings UI persists, profiles switch live |
| 5 | M5 | Palette finds every action, keybinds work |
| 6 | M6 | Quake mode, session restore, search work |
| 7 | M7 | Signed MSIX installs, auto-updates |

---

## Next Steps

1. **Initialize repo** with Cargo workspace + `build.rs` for Zig
2. **Phase 0 vertical slice** — single terminal, no UI chrome
3. **Daily driver test** — use it for real work, fix papercuts
4. **Iterate phases** — each phase ships a usable improvement

---

*Generated with ponytail philosophy: build the minimum that works, delete what isn't needed, question every abstraction.*