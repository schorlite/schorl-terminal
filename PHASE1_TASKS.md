# Phase 1 Task Tracker

**Status Legend:** `[ ]` Todo | `[~]` In Progress | `[x]` Done | `[!]` Blocked

---

## PTY I/O Loop (Foundation)

- [x] **1.1** Add `portable-pty` dependency
  - [x] `Cargo.toml`: add `portable-pty = "0.8"`
  - [x] `src/main.rs`: replace `PtyHandle` stub
  - **Verify:** `cargo build` succeeds ✓

- [x] **1.2** Wire PTY → Terminal (Input)
  - [x] `App::handle_key()` → `PtyHandle::write()`
  - [x] `PtyHandle::write()` → `MasterPty::write()`
  - **Verify:** Type in window → keys reach shell (echoed) ✓ (wired, needs glyph rendering to see)

- [x] **1.3** Wire PTY → Terminal (Output)
  - [x] Background task: `MasterPty::try_clone_reader()` → `Terminal::vt_write()`
  - [x] `App::render()`: drain PTY reader each frame
  - **Verify:** `echo hello` → see output ✓ (wired, needs glyph rendering to see)

---

## Rendering Pipeline (Text on Screen)

- [ ] **1.4** Add `fontdue` + Glyph Atlas
  - [ ] `Cargo.toml`: add `fontdue = "0.7"`
  - [ ] `GlyphAtlas` struct: `Texture2D` + `HashMap<char, GlyphInfo>`
  - [ ] On-demand rasterization + texture upload
  - **Verify:** Window opens, black background (atlas ready)

- [ ] **1.5** Grid Render Pipeline (Instanced Quads)
  - [ ] WGSL shader: vertex = cell pos, instance = glyph_id + colors
  - [ ] Instance buffer: one entry per visible cell
  - [ ] `RenderState` → instance buffer update per frame
  - **Verify:** `ls` → see actual text in grid

- [ ] **1.6** Cursor Rendering
  - [ ] Separate cursor pipeline (single quad)
  - [ ] Read cursor pos/style from `RenderState`
  - [ ] Blink timer (500ms)
  - **Verify:** Blinking block cursor at prompt

---

## Interaction (Usability)

- [ ] **1.7** Selection + Copy
  - [ ] `MouseEncoder` → track selection in `RenderState`
  - [ ] Selection overlay pipeline
  - [ ] Clipboard via `arboard` crate
  - **Verify:** Drag-select → Ctrl+Shift+C → paste in Notepad

- [ ] **1.8** Scrollback Buffer
  - [ ] `TerminalOptions.max_scrollback = 10000`
  - [ ] Mouse wheel / Shift+PgUp → `Terminal.scroll()`
  - [ ] Render viewport + offset slice
  - **Verify:** `seq 1 100` → scroll up → see history

- [ ] **1.9** Dynamic Resize Reflow
  - [ ] `Resized` → `Terminal.resize()` + `pty.resize()`
  - [ ] Glyph atlas + instance buffer recreate
  - **Verify:** Drag window corner → text reflows cleanly

---

## Advanced Features (Polish)

- [ ] **1.10** True Color + Indexed Palette
  - [ ] Per-cell 24-bit RGB from `RenderState`
  - [ ] Shader: `vec3` colors + palette mapping
  - **Verify:** 24-bit color script → smooth gradients

- [ ] **1.11** Kitty Graphics Protocol
  - [ ] Parse Kitty escape codes (libghostty-vt does this)
  - [ ] GPU texture upload for image data
  - [ ] Image pipeline: textured quads at cell positions
  - **Verify:** `kitten icat image.png` → image renders

- [ ] **1.12** Hyperlinks (OSC 8)
  - [ ] `RenderState` exposes hyperlink ranges
  - [ ] Hover → underline + hand cursor
  - [ ] Click → `open::that(url)`
  - **Verify:** Click rendered URL → opens browser

---

## Progress Summary

| Category | Total | Done | In Progress | Remaining |
|----------|-------|------|-------------|-----------|
| PTY I/O | 3 | 3 | 0 | 0 |
| Rendering | 3 | 0 | 0 | 3 |
| Interaction | 3 | 0 | 0 | 3 |
| Advanced | 3 | 0 | 0 | 3 |
| **Total** | **12** | **3** | **0** | **9** |

---

## Usage

1. Open this file in your editor
2. Change `[ ]` to `[~]` when starting a task
3. Change `[~]` to `[x]` when verified working
4. Run `cargo run` after each task to verify
5. Commit after each `[x]` with message: `task 1.N: <description>`

## Next Action

**Task 1.4** — Add `fontdue` + Glyph Atlas for GPU text rendering.