# Tasks

## 1. System Junk Probes Engine

- [x] 1.1 Implement `CleanableTarget` data structure, `PurgeAction` executor, and `src/cleaner/mod.rs` module.
- [x] 1.2 Implement `DockerProbe` to query dangling images, stopped containers, and unused volumes via `docker system df`.
- [x] 1.3 Implement `SnapProbe` to parse disabled revisions from `snap list --all`.
- [x] 1.4 Implement `PackageCacheProbe` measuring global caches for npm (`~/.npm`), pip (`~/.cache/pip`), uv (`~/.cache/uv`), and cargo.
- [x] 1.5 Add unit tests in `src/cleaner/mod.rs` verifying probe outputs, empty fallbacks, and error resilience.

## 2. Virtual Scrapyard Avenue & Navigation

- [x] 2.1 Implement `build_scrapyard_avenue() -> BuildingNode` mapping detected junk targets into a virtual street node with logarithmic heights.
- [x] 2.2 Add `--scrapyard` CLI flag to `src/main.rs` and `src/bin/gui.rs` to launch directly into the Scrapyard Avenue.
- [x] 2.3 Wire `[J]` key toggle in `src/app.rs` and `src/gui/mod.rs` to transition seamlessly between filesystem streets and the Scrapyard Avenue.
- [x] 2.4 Add unit tests in `src/app.rs` validating Scrapyard mode toggling, breadcrumbs, and `[Esc]` return transitions.

## 3. Targeted Demolition & Bulldozer Batch Wipe

- [x] 3.1 Extend `src/demolish.rs` to support `PurgeAction` execution for individual Scrapyard buildings, running non-blocking cleanup.
- [x] 3.2 Implement Bulldozer batch wipe (`[C]`) prompt and sequential execution loop in `src/app.rs` and `src/gui/mod.rs`.
- [x] 3.3 Add localized Scrapyard notifications, HUD hints, and demolition banners in `src/i18n.rs`.
- [x] 3.4 Add unit tests for individual target purging and batch wipe state transitions.

## 4. Visual Styling, Verification & Documentation

- [x] 4.1 Add industrial scrapyard atmosphere styling, `[SCRAP]` neon signboards, and street nameplates in `src/gui/render.rs`.
- [x] 4.2 Run `cargo test` to verify all unit and integration tests pass cleanly.
- [x] 4.3 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` to enforce strict code quality.
- [x] 4.4 Run smoke verification of Scrapyard Avenue on X11 and update `README.md` documentation.
