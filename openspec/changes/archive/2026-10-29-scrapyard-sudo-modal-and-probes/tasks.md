# Tasks

## 1. Data Model & PurgeAction Elevation

- [x] 1.1 Add `requires_sudo: bool` to `BuildingNode` in `src/model.rs`, decoupling it from `is_system_protected`, and verify model unit tests pass with `cargo test model`.
- [x] 1.2 Implement `execute_with_sudo(&self, password: &str) -> Result<(), String>` in `src/cleaner/mod.rs` using `sudo -S` with piped stdin, verifying password zeroing and error capture with unit tests.
- [x] 1.3 Add new `PurgeAction` variants for journal vacuuming (`journalctl`) and multi-path directory cleanup, verifying variant execution with unit tests in `src/cleaner/mod.rs`.

## 2. Extended System Junk Probes

- [x] 2.1 Implement `probe_snap_user_profiles()` in `src/cleaner/mod.rs` to quantify obsolete `~/snap/<app>/<old_rev>` profiles and verify with unit tests.
- [x] 2.2 Implement `probe_system_journal()` in `src/cleaner/mod.rs` to check `/var/log/journal` via `journalctl --disk-usage` and verify with unit tests.
- [x] 2.3 Implement `probe_cockpit_updates()` and `probe_codex_runtimes()` in `src/cleaner/mod.rs` and verify with unit tests.
- [x] 2.4 Implement `probe_gradle_wrappers()` in `src/cleaner/mod.rs`, wire all new probes into `survey_all_system_junk()`, and verify with `cargo test cleaner`.

## 3. Ratatui TUI Sudo Password Modal

- [x] 3.1 Create `SudoModalState` in `src/demolish.rs` supporting masked character input (`*`), Backspace, Enter, Esc, and error display, verified by unit tests.
- [x] 3.2 Implement `render_sudo_modal()` in `src/demolish.rs` with styled Ratatui borders and privilege warnings, verified by render tests.
- [x] 3.3 Connect TUI key handling in `src/app.rs` to trigger `SudoModalState` when demolishing targets with `requires_sudo = true`, verifying execution and state transitions.

## 4. Macroquad GUI Sudo Password Modal

- [x] 4.1 Define `GuiSudoModal` in `src/gui/state.rs` tracking target index, password string buffer, and error message state.
- [x] 4.2 Implement `render_sudo_modal()` in `src/gui/render/overlay.rs` matching the pixel-art retro HUD theme with masked asterisks.
- [x] 4.3 Update GUI demolition trigger and event loop in `src/gui/mod.rs` to open `GuiSudoModal` for elevated targets, execute `execute_with_sudo()`, and refresh capacity indicators.

## 5. Bulldozer Batch Purge & End-to-End Verification

- [x] 5.1 Update `execute_bulldozer_purge()` in `src/demolish.rs` to accept optional superuser authentication, purging elevated items if supplied or skipping them if omitted.
- [x] 5.2 Connect Bulldozer hotkey `[C]` in `src/app.rs` and `src/gui/mod.rs` to offer a single sudo password prompt when elevated items are present in the avenue.
- [x] 5.3 Run full test suite with `cargo test` and verify that both user-level and elevated junk categories behave correctly.
