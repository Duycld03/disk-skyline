# Design

## Context

The 2D arcade GUI (`src/gui/mod.rs` and `src/gui/render/`) uses Macroquad to process user keyboard input. `A` and `D` are standard for left and right avatar walking, but `D` was also configured as a secondary shortcut for demolition alongside `Delete`. 

In `src/cleaner/mod.rs`, `CleanableTarget` already tags items with `requires_sudo: bool`. However, `execute_bulldozer_purge` in `src/demolish.rs` iterates over every child in the Scrapyard Avenue without checking `requires_sudo`, running `snap remove` or `apt-get clean` and thereby invoking PolicyKit GUI password prompts sequentially.

Furthermore, developer caches like `pip` (`~/.cache/pip`, 2.7 MB) are dropped due to a 5 MB filter threshold, while modern Python environments managed by `uv` store tools and interpreter artifacts in `~/.local/share/uv` (512 MB) rather than `~/.cache/uv`.

## Goals / Non-Goals

**Goals:**
- Completely decouple horizontal WASD movement (`A`/`D`) from demolition (`X` / `Delete`), eliminating unintended demolition modal popups while strolling.
- Make Bulldozer Clean All (`[C]`) zero-interruption by executing only user-level, non-elevated cleanups (`requires_sudo == false`).
- Safely handle administrative targets (`requires_sudo == true`) by locking them on the Scrapyard Avenue and displaying clean, copyable terminal commands (e.g. `sudo snap remove ...`).
- Expand system junk discovery to detect Pip caches down to 1 MB, Python `uv` toolchain storage in `~/.local/share/uv`, and user-owned `/tmp` clutter.

**Non-Goals:**
- Injecting an interactive `sudo` password prompt or terminal emulator into the Macroquad OpenGL canvas.
- Forcibly deleting other system users' files in `/tmp`.

## Decisions

### 1. Keybinding Architecture: WASD Stroll vs `[X]` Demolition
- **Decision**: Rebind single-target demolition in GUI to `KeyCode::X` and `KeyCode::Delete`. Remove all `KeyCode::D` triggers from demolition.
- **Rationale**: `D` is universally used for moving right in 2D platformers and arcade strolls. Mapping demolition to `X` (eXplode / eXterminate) fits retro arcade conventions and completely frees `A`/`D` for fluid movement.
- **Alternatives Considered**: Keeping `Shift+D` / `Ctrl+D` was tried, but users still found `[D] Demolish` listed on the HUD conflicting with `[A/D] Walk`.

### 2. Sudo Safety Guard in Bulldozer Batch Execution
- **Decision**: In `src/demolish.rs::execute_bulldozer_purge`, check `requires_sudo` before executing any action. If `requires_sudo` is `true`, skip automated execution and leave the building on the avenue with an informational note.
- **Rationale**: Prevents Polkit from spawning sequential modal dialogs for each disabled snap revision or apt cache. Users can safely hit `[C]` knowing it will only purge Docker, Pip, UV, Cargo, and user caches silently and instantaneously.

### 3. Comprehensive Python & Pip Probing
- **Decision**: Lower Pip threshold to 1 MB. Add dedicated probe `probe_uv_data()` checking `~/.local/share/uv` (`tools/` and `python/`), purging via `uv cache clean` or targeted directory removal.
- **Rationale**: Modern Python developers frequently use `uv` which places hundreds of megabytes in `~/.local/share/uv` rather than `~/.cache/pip`.

### 4. User-Owned `/tmp` Clutter Detection
- **Decision**: Add `probe_tmp_directory()` scanning entries in `/tmp` owned by the current user UID older than 24 hours or matching temporary tool patterns (`test_*`, `tmp*`).
- **Rationale**: `/tmp` frequently accumulates gigabytes of abandoned build and test artifacts that users can safely purge without administrative rights.

## Risks / Trade-offs

- **Snap & Apt Retention in Scrapyard**: Because Bulldozer skips sudo targets, disabled Snap revisions won't be cleared when pressing `[C]`. 
  - *Mitigation*: The HUD inspection card and status banner will explicitly print the exact command (e.g. `Run in terminal: sudo snap remove <pkg> --revision <rev>`). This avoids PolicyKit popups while giving full control to the user.
