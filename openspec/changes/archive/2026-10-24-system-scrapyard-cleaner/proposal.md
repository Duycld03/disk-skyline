# Proposal: System Scrapyard Cleaner

## Why

Developer workstations rapidly accumulate tens of gigabytes of reclaimable environment junk—including disabled Snap revisions, dangling Docker images and volumes, package manager caches (npm, pip, uv, cargo), and build logs—causing frequent disk exhaustion. Instead of requiring users to manually diagnose, remember, and execute disparate CLI purge commands, Disk Skyline should provide an integrated, game-inspired "Virtual Scrapyard Avenue" where system junk is surveyed, visualized as explorable cyberpunk skyline buildings, and safely purged either individually with particle explosions or in batch using a bulldozer wipe.

## What Changes

- **System Junk Probing Engine**: Introduce modular, non-destructive system junk probes to calculate exact reclaimable bytes across:
  - Docker dangling images, stopped containers, and unused volumes (`docker system df`).
  - Disabled Snap package revisions (`snap list --all`).
  - Package manager global caches (`~/.npm/_cacache`, `~/.cache/pip`, `~/.cache/uv`, `~/.cargo/registry/cache`, `~/.cache/ms-playwright`).
  - System package archives and journal logs (`/var/cache/apt/archives`, journal vacuum).
- **Virtual Scrapyard Avenue (Đại Lộ Phế Liệu)**:
  - Hotkey `[J]` (or CLI flag `--scrapyard`) shifts the active avenue to a virtual street where each junk category is rendered as a logarithmic skyscraper with a distinct industrial `[SCRAP]` / `[JUNK]` aesthetic.
  - Interactive street navigation with real-time avatar tracking and street nameplates displaying itemized reclaimable space.
- **Selective & Batch Demolition (Clean All / Bulldozer)**:
  - Individual demolition `[D]`: Allows users to walk up to a specific junk building (e.g., Docker Images or NPM Cache) and demolish it individually with particle debris explosion physics.
  - Bulldozer batch wipe `[C]`: Prompts a confirmation dialog to sequentially purge all surveyed junk categories, triggering chain explosion effects and dynamically updating the global disk capacity gauge.
  - Clean separation of user-level purges from root/sudo purges (with graceful guidance and safety locks).
- **Bilingual & Dual-Interface Support**:
  - Full support across both Terminal TUI and 2D Game GUI with seamless runtime language toggling (`[L]`).

## Capabilities

### New Capabilities
- `system-cleaner`: Probing, calculating, presenting, and safely purging system-wide developer caches and environment junk.

### Modified Capabilities
- `game-gui`: Adds rendering support for the Virtual Scrapyard Avenue, industrial junk facade styling, and sequential batch explosion effects.
- `disk-skyline`: Adds the `--scrapyard` CLI argument and hotkey routing between regular directories and the virtual scrapyard avenue.

## Impact

- Zero data loss: All probes target disposable caches and dangling artifacts that can be safely rebuilt or refetched.
- Clean architectural boundary: Probes execute non-blocking checks and delegate actual deletion to platform-appropriate commands (`docker system prune`, `rm -rf`, `snap remove`).
