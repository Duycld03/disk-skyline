# Proposal

## Why

Terminal-based UI tools (TUI) like `top` or `htop` are restricted by fixed terminal character grids, lack smooth 60 FPS animations, and cannot render true 2D pixel-art graphics. Introducing a standalone 2D desktop game window powered by Macroquad—while keeping the lightweight CLI/TUI binary intact—transforms disk visualization into an interactive retro arcade game with walking avatars, smooth camera panning, and RPG-style HUDs.

## What Changes

- **Dual Binary Architecture**:
  - Keep `disk-skyline` as the fast, zero-GUI terminal CLI/TUI tool for servers and SSH.
  - Introduce `disk-skyline-gui` as the native desktop 2D pixel game window powered by Macroquad.
  - Both binaries share 100% of core engine logic (`model`, `scanner`, `skyline`, `i18n`, `system_protection`, `demolish`).
- **Macroquad 2D Game Window Frontend**:
  - Native cross-platform window (Windows `.exe`, Linux X11/Wayland, macOS) at 1280x720 base resolution with fullscreen toggle (`F11`).
  - 60 FPS game loop with smooth camera horizontal scrolling.
  - **Pixel Walking Avatar**: 2D animated avatar strolling down the avenue, turning left/right, and jumping with `Space`.
  - **Pixel Skyline Visuals**: Thematic retro building facades (Offices, Tech Centers, Media Centers, Archives, Restricted Bunkers, System Locked structures) rendered with pixel shading, night sky stars, and lampposts.
  - **RPG Style Game HUD**: Disk free capacity rendered as a retro Health/Mana bar (`[████████░░░░]`), boss encounter stat cards for large directories, and arcade command prompts.
  - **Bilingual Support**: Dynamic language toggle (`L` key) switching between Vietnamese and English.
  - **Particle Effects**: Pixel debris explosion animation when demolishing buildings.
- **Cross-Platform Release Automation**:
  - Update GitHub Actions workflows (`release.yml` and `ci.yml`) to compile and package both `disk-skyline` and `disk-skyline-gui` for Linux, Windows, and macOS.

## Capabilities

### New Capabilities
- `game-gui`: Native 2D graphical game window frontend powered by Macroquad, featuring real-time 60 FPS rendering, walking pixel avatar, retro RPG HUD stats, and interactive filesystem exploration.

### Modified Capabilities
- None (existing `disk-skyline` TUI, `i18n`, and `system-protection` specs remain preserved and authoritative).

## Impact

- **Affected Code**: `Cargo.toml` (add Macroquad dependency), new `src/bin/gui.rs` (or `src/gui/` module), `src/lib.rs` (exporting core interfaces), `.github/workflows/ci.yml`, `.github/workflows/release.yml`.
- **Breaking Changes**: None. The terminal CLI `disk-skyline` remains 100% backward compatible.
