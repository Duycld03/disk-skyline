# Tasks

## 1. Keybinding Decoupling & GUI Demolition Hotkey Update

- [x] 1.1 In `src/gui/mod.rs`, rebind demolition trigger to `KeyCode::X` and `KeyCode::Delete`, removing all `KeyCode::D` checks from demolition triggers so `A`/`D` control horizontal walking without conflict.
- [x] 1.2 In `src/app.rs`, accept `KeyCode::Char('x')` alongside `KeyCode::Char('d')` and `KeyCode::Delete` for demolition modal opening in TUI.
- [x] 1.3 In `src/gui/render/overlay.rs`, `src/ui.rs`, and `src/i18n.rs`, update HUD hints and localized command bar strings to display `[A/D] Walk  [X] Demolish  [C] Bulldozer`.
- [x] 1.4 Add unit tests in `src/gui/state.rs` and `src/app.rs` validating that `X` triggers demolition and `D` is reserved strictly for walking.

## 2. Sudo Safety Guard in Bulldozer Batch Execution

- [x] 2.1 In `src/demolish.rs::execute_bulldozer_purge`, enforce non-elevated safety: execute only cleanable targets with `requires_sudo == false` (e.g. Docker, Pip, UV, Cargo, NPM, Temp), preserving targets with `requires_sudo == true` on the avenue.
- [x] 2.2 In `src/cleaner/mod.rs`, provide formatted manual terminal guidance for sudo targets (e.g. `sudo snap remove <pkg> --revision <rev>` and `sudo apt-get clean`).
- [x] 2.3 In `src/gui/render/overlay.rs` and `src/app.rs`, update Scrapyard HUD inspection cards for `[SYS]` targets to clearly state manual execution instructions.
- [x] 2.4 Add unit tests in `src/demolish.rs` verifying that `execute_bulldozer_purge` safely skips sudo targets without invoking Polkit dialogs or erroring out.

## 3. Python/Pip and Extended System Junk Probes

- [x] 3.1 In `src/cleaner/mod.rs::probe_package_caches`, lower the Pip cache size threshold from 5 MB to 1 MB so smaller developer caches (e.g. 2.7 MB) are visualised.
- [x] 3.2 Implement `probe_uv_data()` scanning `~/.local/share/uv` (`tools/` and `python/`) and quantifying reclaimable Python runtimes and tool environments.
- [x] 3.3 Implement `probe_tmp_directory()` measuring user-owned clutter in `/tmp` when exceeding 50 MB with a safe directory purge action.
- [x] 3.4 Wire the new probes into `survey_all_system_junk()`.
- [x] 3.5 Add unit tests in `src/cleaner/mod.rs` verifying the lowered Pip threshold, UV probe, and `/tmp` probe with empty fallback resilience.

## 4. Verification, Visual Smoke Testing & Documentation

- [x] 4.1 Run `cargo test` verifying all unit and integration tests pass cleanly.
- [x] 4.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` to enforce zero warnings and clean formatting.
- [x] 4.3 Run smoke verification on X11 testing walking with `D`, demolishing with `X`, and triggering Bulldozer without PolicyKit popup prompts.
- [x] 4.4 Update `README.md` controls and documentation with the updated hotkeys and expanded junk targets.
