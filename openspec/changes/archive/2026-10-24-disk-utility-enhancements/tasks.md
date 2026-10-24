# Tasks

## 1. Core Data & External Opener

- [x] 1.1 Add `modified_secs_ago: Option<u64>` and staleness evaluation (`is_stale(&self) -> bool`) to `BuildingNode` and update `scanner.rs`.
- [x] 1.2 Implement detached cross-platform file/folder opener `launch_default_app(path: &Path)` in `src/app.rs`.
- [x] 1.3 Bind `o` key in `App::handle_key` to launch currently selected target and emit localized feedback message.
- [x] 1.4 Add unit tests in `src/app.rs` and `src/model.rs` for file launching and staleness logic.

## 2. Spotlight Search Engine

- [x] 2.1 Add `search_query: Option<String>` buffer and search navigation methods (`start_search`, `update_search`, `cancel_search`) to `App`.
- [x] 2.2 Implement `/` key handling and typing input loop in terminal TUI (`src/app.rs` and `src/ui.rs`).
- [x] 2.3 Implement `/` key handling, search text entry, and smooth camera pan lerping in 2D GUI (`src/gui/mod.rs`).
- [x] 2.4 Add unit tests for search matching and index jumping behavior.

## 3. GUI Visual Enhancements

- [x] 3.1 Implement 14px street radar minimap bar in `src/gui/render.rs` showing building profile and real-time avatar marker.
- [x] 3.2 Implement search bar input overlay rendering in `src/gui/render.rs` when spotlight search is active.
- [x] 3.3 Apply staleness styling in `src/gui/render.rs` (dimmed window brightness and `[OLD]` indicator) for files older than 180 days.

## 4. Quality Verification & Documentation

- [x] 4.1 Run `cargo test` to verify all unit and integration tests pass cleanly.
- [x] 4.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` to enforce strict code quality.
- [x] 4.3 Run smoke verification of new GUI features on X11 and update `README.md` documentation.
