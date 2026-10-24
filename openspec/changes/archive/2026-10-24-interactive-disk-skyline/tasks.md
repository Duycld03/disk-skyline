# Tasks

## 1. Project Scaffolding & Core Types

- [x] 1.1 Initialize Rust Cargo binary project `disk-skyline` with `Cargo.toml` dependencies (`ratatui`, `crossterm`, `clap`, `walkdir`, `rayon`) and verify `cargo check` compiles successfully
- [x] 1.2 Implement core data structures (`BuildingNode`, `BuildingCategory`, `NavigationState`, `SortMode`) in `src/model.rs` and verify unit tests validate node instantiation and category matching

## 2. Bounded In-Memory Disk Scanner

- [x] 2.1 Implement multi-threaded bounded filesystem walker in `src/scanner.rs` with configurable depth limits (default 6) and recursive directory size accumulation, verifying unit tests on fixture directories
- [x] 2.2 Add symlink recursion protection and dotfile classification (`is_hidden`) in `src/scanner.rs`, verifying unit tests confirm `.git` and hidden files are flagged as restricted

## 3. Logarithmic Skyline Engine & Facade Styler

- [x] 3.1 Implement logarithmic building height scaling function in `src/skyline.rs` and verify unit tests confirm small (10 KB) and large (10 GB) files both produce valid viewport row heights
- [x] 3.2 Implement building facade rendering in `src/render.rs` with roof styles, windows, and category themes (including bunker/hazard patterns for hidden items), verifying terminal buffer rendering tests

## 4. Interactive TUI Navigation & Camera Viewport

- [x] 4.1 Implement horizontal camera panning (`h`/`l` / arrows) and minimap widget in `src/app.rs`, verifying navigation state updates correctly when cycling through buildings
- [x] 4.2 Implement drill-down directory entry (`Enter`) and parent return (`Esc`/`Backspace`) with breadcrumbs in `src/app.rs`, verifying navigation stack transitions between parent and child directories
- [x] 4.3 Implement HUD metadata inspection bar at screen bottom in `src/ui.rs`, displaying focused building details (path, size, item count, classification)
- [x] 4.4 Implement on-demand manual rescan (`r` / `F5`) in `src/app.rs`, refreshing the active directory subtree from disk and updating the in-memory tree

## 5. Safe Demolition System

- [x] 5.1 Implement demolition modal dialog in `src/demolish.rs` with two-tier confirmation (`y` for standard items, `YES` for restricted dotfiles or items > 1 GB)
- [x] 5.2 Implement filesystem deletion and immediate in-memory tree pruning with parent size adjustment in `src/demolish.rs`, verifying unit tests confirm tree state is updated without full disk rescan

## 6. CLI Wiring & End-to-End Verification

- [x] 6.1 Wire `clap` CLI arguments in `src/main.rs` to support optional target directory and `--depth` flag, displaying an initial loading splash while scanning
- [x] 6.2 Run full test suite (`cargo test`) and perform an interactive TUI smoke run verifying camera movement, drill-down, bunker styling, and demolition dialog
