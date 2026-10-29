# Proposal

## Why

Disk Skyline's Scrapyard avenue currently detects only a limited set of developer caches and completely blocks demolition of high-impact system junk (such as Snap disabled revisions and APT archives) with a passive terminal notice. This occurs because the codebase conflates `requires_sudo` with `is_system_protected`. Users facing critical disk constraints cannot clean significant system debris (such as multiple gigabytes of obsolete Snap revision profiles, systemd journal logs, and heavy developer runtimes) without exiting the application. Providing automated detection for these real-world clutter targets and an interactive In-App Sudo Password Modal enables seamless, secure privilege elevation directly within both the 2D GUI and TUI.

## What Changes

- **New System Junk Probes**:
  - Probe obsolete Snap application revision profiles in user home (`~/snap/<app>/<old_rev>`) left behind after snap updates (user-level, no sudo required).
  - Probe systemd journal logs in `/var/log/journal` via `journalctl --vacuum-size=100M` (requires sudo).
  - Probe accumulated Cockpit tools update archives in `~/.local/share/cockpit-tools/updates` (user-level).
  - Probe Codex runtime dependency caches in `~/.cache/codex-runtimes` (user-level).
  - Probe Gradle wrapper distribution downloads in `~/.gradle/wrapper/dists` (user-level).
- **Decouple Protection from Privilege Elevation**:
  - Reclassify `is_system_protected` strictly for immutable OS system infrastructure (`/`, `/bin`, `/usr`, `/etc`, `/boot`).
  - Retain `requires_sudo = true` on cleanable targets without marking their building nodes as system-protected, replacing the lock icon with an elevated privilege indicator.
- **In-App Sudo Password Modal**:
  - Introduce an interactive password prompt modal rendered natively in both Ratatui (TUI) and Macroquad (GUI) styled with the game's retro aesthetic.
  - Mask password input (`****`), wipe buffers from memory after use, and execute elevated actions via `sudo -S` over piped `stdin`.
  - Display clear error messages and allow retry or cancellation when an incorrect password is provided.
- **Enhanced Bulldozer Batch Purge**:
  - Allow Bulldozer (`[C]`) to process elevated targets by prompting for the sudo password once, or skipping elevated targets gracefully if authorization is canceled or denied.

## Capabilities

### Modified Capabilities
- `system-cleaner`: Add requirements for extended junk survey probes, in-app sudo elevation password authentication, and elevated demolition execution.
- `system-protection`: Decouple operating system critical directory protection from cleanable targets requiring privilege elevation.

## Impact

- `src/cleaner/mod.rs`: New probe functions, expanded `PurgeAction` execution paths supporting password-piped `sudo -S`.
- `src/model.rs`: Add `requires_sudo` property to `BuildingNode`; ensure `is_system_protected` is false for elevated cleanable junk.
- `src/demolish.rs`: Add TUI Sudo password modal state, keystroke masking, and elevated purge invocation.
- `src/gui/`: Add Macroquad GUI Sudo password modal dialog, text input event handling, and rendering.
- `src/app.rs`: Connect demolition triggers for elevated targets to the password modal flow instead of rejecting with a static notice.
