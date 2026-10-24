# Tasks

## 1. Dual Binary Cargo & Dependency Setup

- [x] 1.1 Add `macroquad = "0.4"` dependency to `Cargo.toml`.
- [x] 1.2 Define `[[bin]]` sections in `Cargo.toml` for `disk-skyline` (`src/main.rs`) and `disk-skyline-gui` (`src/bin/gui.rs`).
- [x] 1.3 Verify existing test suite and `disk-skyline` TUI continue building and passing cleanly.

## 2. 2D Game Architecture & Avatar Physics Engine

- [x] 2.1 Scaffold `src/bin/gui.rs` with Macroquad window configuration (`1280x720`, resizable) and `GameApp` state.
- [x] 2.2 Implement avatar physics (horizontal walking with `A`/`D` and Arrow keys, jumping on `Space`, directional facing).
- [x] 2.3 Implement smooth horizontal camera tracking lerping toward the avatar's position.

## 3. Pixel-Art Skyline & Thematic Facade Rendering

- [x] 3.1 Implement background starfield rendering and ground street sidewalk line.
- [x] 3.2 Implement pixel building facades with logarithmic scaling, thematic category colors, windows, and doorways.
- [x] 3.3 Implement 2D pixel avatar sprite rendering reflecting movement and facing direction.

## 4. Retro Arcade HUD & Bilingual Support

- [x] 4.1 Render top Game HUD with disk free space capacity bar (`[████████░░░░]`) and avenue breadcrumbs.
- [x] 4.2 Render bottom RPG Target Card showing focused building metadata, category badge, and control key hints.
- [x] 4.3 Wire `L` key to dynamically switch all HUD texts between Vietnamese and English.

## 5. Interactive Navigation, Demolition Particles & CI/CD

- [x] 5.1 Implement directory drill-down on `Enter` and parent navigation on `Backspace`/`Esc`.
- [x] 5.2 Implement safe demolition on `d` with confirmation modal and pixel debris explosion particle effects.
- [x] 5.3 Update GitHub Actions `.github/workflows/ci.yml` and `.github/workflows/release.yml` to build and package both binaries for Linux, Windows, and macOS.
