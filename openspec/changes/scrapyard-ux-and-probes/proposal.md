# Proposal

## Why

In the 2D Game GUI, key `D` is currently bound to both rightward walking (WASD convention) and building demolition, causing jarring input collisions and user confusion during avatar strolls. Additionally, running Bulldozer batch cleanup (`[C]`) attempts to purge system-level targets requiring `sudo` (such as disabled Snap revisions), which triggers repetitive PolicyKit desktop password dialogs. Finally, the current system junk probes overlook developer pip caches below 5MB, modern `uv` Python caches in `~/.local/share/uv`, and `/tmp` temporary build artifacts.

## What Changes

- **WASD & Demolition Key Decoupling**: Rebind single-building demolition in the 2D Game GUI to `[X]` (eXplode / Destroy) and `[Delete]`, dedicating `[A]` and `[D]` exclusively to left/right walking.
- **Bulldozer Sudo Safety Guard**: Restrict Bulldozer Clean All (`[C]`) to user-level safe targets that do not require administrative elevation (Docker dangling containers/volumes, package caches for Pip/UV/NPM/Cargo, and temporary files). Targets requiring root/sudo privileges (Snap revisions, APT archives) are clearly badged as System Lock (`[SYS]`) and provide safe manual terminal command recommendations instead of triggering PolicyKit popup spam.
- **Lowered Pip Cache Threshold**: Lower Pip cache detection threshold from 5 MB to 1 MB, ensuring developer wheel caches (such as 2.7 MB) are accurately detected and visualised as Scrapyard buildings.
- **UV Tool & Python Runtime Probe**: Add probe for `~/.local/share/uv` detecting installed Python toolchains and cached environments.
- **Temporary Directory Probe**: Add probe for `/tmp` user-owned debris when reclaimable size exceeds 50 MB.
- **Updated Bilingual HUD & Hotkey Hints**: Update all HUD tooltips, control cards, and bilingual descriptions (VI/EN) across both TUI and GUI to reflect `[X]` Demolish, `[A]/[D]` Walk, and `[C]` Bulldozer.

## Capabilities

### Modified Capabilities
- `game-gui`: Rebind demolition hotkey from `[D]` to `[X]` / `[Delete]`, remove input collisions with `[D]` movement, and update Scrapyard HUD hotkey prompts.
- `system-cleaner`: Add non-elevated Sudo Safety Guard to Bulldozer batch purge, lower Pip cache detection threshold to 1 MB, and introduce probes for `uv` and `/tmp`.
- `disk-skyline`: Align TUI demolition keybindings to accept `[x]` / `[Del]` alongside existing controls and update localized command bar hints.
