# Design

## Context

See `proposal.md` for background. The existing codebase compiles to a single binary `disk-skyline` providing an interactive terminal UI via Ratatui and Crossterm. We are introducing a 2D desktop game frontend powered by Macroquad while retaining the existing terminal binary unchanged.

## Goals / Non-Goals

**Goals:**
- Implement a standalone native desktop game window (`disk-skyline-gui`) running at 60 FPS using Macroquad.
- Preserve the existing `disk-skyline` terminal CLI/TUI binary with zero regressions.
- Share 100% of underlying filesystem models, multi-threaded scanners, logarithmic height calculations, OS protection guards, and i18n dictionaries from `disk_skyline` library crate.
- Implement an animated pixel avatar with smooth walking physics, directional facing, and jumping.
- Render pixel-art building facades, starry night skies, and a retro RPG Game HUD with disk capacity HP bars.
- Cross-platform support for Windows (.exe), Linux (X11/Wayland), and macOS.

**Non-Goals:**
- Merging GUI and TUI into a single tangled binary (keeping them as clean dual binaries prevents GUI dependencies from polluting server TUI environments).
- Complex 3D rendering or realistic shaders (pure 2D retro pixel arcade style).

## Decisions

### 1. Dual Binary Structure in Cargo

**Decision:** Define two distinct binary targets in `Cargo.toml`:
- `[[bin]] name = "disk-skyline" path = "src/main.rs"` (Existing CLI/TUI).
- `[[bin]] name = "disk-skyline-gui" path = "src/bin/gui.rs"` (New 2D Game Window).
**Rationale:**
- Zero impact on the existing terminal CLI: server users can continue building `cargo build --bin disk-skyline` without any graphical libraries.
- Windows desktop users can download and double-click `disk-skyline-gui.exe` directly.

### 2. Macroquad Framework Integration

**Decision:** Add `macroquad = "0.4"` to `Cargo.toml`.
**Rationale:**
- Pure Rust windowing and OpenGL/DirectX abstraction via `miniquad`.
- Native Windows support without requiring C runtime or external DLLs.
- Simple, declarative immediate-mode 2D drawing API (`draw_rectangle`, `draw_text`, `draw_circle`, `draw_line`).
- Standard game loop with `next_frame().await`.

### 3. Game Loop & Camera Architecture (`src/bin/gui.rs`)

**Decision:** Build a dedicated `GameApp` state holding:
- `root_node: BuildingNode`: Active in-memory filesystem tree.
- `current_path_indices: Vec<usize>`: Navigation hierarchy breadcrumbs.
- `avatar: Avatar`: Tracks `x`, `y`, `vx`, `vy`, `grounded: bool`, and `facing: Direction`.
- `camera_x: f32`: Smoothly lerps toward `avatar.x` (target camera position = `avatar.x - screen_width / 2.0`).
- `particles: Vec<DebrisParticle>`: Particle simulation for demolition effects.
- `language: Language`: Dynamic language state toggleable via `L`.
- `selected_index: usize`: Automatically resolves to whichever building the avatar stands in front of.

### 4. Pixel Art Visual Representation

**Decision:**
- **Skyline Canvas**: Ground street line drawn at `screen_height - 180.0`. Building heights scaled logarithmically between 40px and `screen_height - 240.0`.
- **Facades**: Rectangles drawn with category-specific colors and pixel patterns (windows and doorways). Protected buildings carry a padlock badge and security barrier.
- **Walking Avatar**: Drawn as a 3-tier retro pixel character. Direction determines facing eyes and feet offset.
- **Game HUD Overlay**:
  - Top Bar: System disk free space rendered as an arcade Health/Mana bar (`[████████░░░░]`).
  - Bottom Bar: RPG style Target Card displaying focused building metadata, category badge, and control hotkeys.

### 5. Cross-Platform CI/CD Updates

**Decision:** Update `.github/workflows/ci.yml` and `release.yml`:
- On Ubuntu runners, install `libx11-dev libxi-dev libgl1-mesa-dev` so Macroquad can link X11/GL headers.
- On Windows and macOS runners, zero extra setup is required.
- Package both `disk-skyline` and `disk-skyline-gui` into release archives.

## Risks / Trade-offs

- **Risk:** Headless Linux environments lack display servers (`DISPLAY` not set).
  - *Mitigation:* The GUI binary detects if window creation fails and displays a clean message suggesting the terminal version `disk-skyline`.
- **Risk:** Compile times increase slightly due to Macroquad.
  - *Mitigation:* Macroquad is based on `miniquad`, which compiles in under 15 seconds, unlike Bevy or WGPU.
