# Design: System Scrapyard Cleaner

## Context

See `proposal.md` for background and problem motivation. Disk Skyline currently models filesystems as in-memory `BuildingNode` hierarchies, with rendering engines (TUI and 2D Macroquad) drawing buildings with logarithmic scaling and handling drill-down, pan, and demolition (`execute_demolition`). To clean system caches without inventing a separate, disjoint UI, the Scrapyard survey can be modeled as a synthetic `BuildingNode` ("Virtual Scrapyard Avenue") populated with cleanable system junk targets as child nodes.

## Goals / Non-Goals

**Goals:**
- Provide fast, non-blocking probes (< 500ms total) to detect cleanable bytes across Docker, Snap revisions, package caches (npm, pip, uv, cargo), and system archives.
- Model cleanable targets as first-class `BuildingNode` items so all existing camera, avatar, logarithmic scaling, street nameplates, and particle explosion mechanics work out-of-the-box.
- Support both individual demolition (`[D]`) for surgical cleanup and a bulldozer batch wipe (`[C]`) for one-click recovery.
- Ensure safe execution with clear bilingual feedback and zero risk of unintended data loss.

**Non-Goals:**
- Sweeping workspace source code or active git repositories (reserved for standard directory navigation).
- Building an OS package manager or replacing tools like `apt` or `snap` for installation.
- Running unauthenticated root exploits: privileged operations (Snap, Apt) must require explicit confirmation or escalate gracefully.

## Decisions

### 1. Synthetic Scrapyard Node (`BuildingNode`)
- **Decision**: Model the Scrapyard Avenue as a synthetic root `BuildingNode` where each child represents a surveyed junk category:
  ```rust
  pub struct CleanableTarget {
      pub id: &'static str,
      pub name: String,
      pub size: u64,
      pub description: String,
      pub purge_command: PurgeAction,
      pub requires_sudo: bool,
  }
  ```
- **Rationale**: Reuses 100% of existing skyline layout, logarithmic height calculation, avatar stroll, camera panning, and particle demolition physics without duplicate rendering pipelines.
- **Alternatives considered**: A separate modal dialogue or table list. Rejected because it breaks game immersion and fails to visually convey relative junk sizes compared to each other.

### 2. Modular Probe Architecture (`src/cleaner/mod.rs`)
- **Decision**: Implement independent probes:
  - `DockerProbe`: Calls `docker system df --format '{{json .}}'` with a 500ms timeout.
  - `SnapProbe`: Parses `snap list --all` for `disabled` revisions.
  - `PackageCacheProbe`: Measures `~/.npm/_cacache`, `~/.cache/pip`, `~/.cache/uv`, and `~/.cargo/registry/cache`.
  - `SystemCacheProbe`: Measures `/var/cache/apt/archives` and systemd journals.
- **Rationale**: If a user does not have Docker or Snap installed, that probe immediately yields `None` or 0 bytes without slowing down the application or throwing errors.

### 3. Demolition Action Delegation
- **Decision**: When demolishing a Scrapyard building, instead of standard `std::fs::remove_dir_all`, the demolition engine dispatches the corresponding `PurgeAction` (e.g. `docker system prune -f`, removing cache directories, or invoking snap cleanup).
- **Rationale**: Ensures the proper system tools handle their own internal state (e.g., Docker updating its image metadata) while Skyline verifies the reclaimed bytes.

### 4. Bulldozer Sequence (`[C]`)
- **Decision**: Pressing `[C]` on the Scrapyard Avenue prompts a batch confirmation modal. Upon confirmation, it executes purges across all detected targets sequentially, triggering chain explosion particles in the GUI and updating the global disk capacity gauge.

## Risks / Trade-offs

- **Risk: Privileged Snap/Apt Cleanup**: Purging snap revisions requires root privileges.
  - *Mitigation*: Mark these targets with `[SUDO]` in the HUD/signboard. If run unprivileged, execute via `pkexec` or emit the exact copy-paste command for the user, while proceeding with user-level purges (Docker, caches) without blocking.
- **Risk: Slow Docker CLI on Unresponsive Daemons**: If the Docker daemon is hung, CLI calls could block.
  - *Mitigation*: Spawn probes with a strict 1-second process timeout and capture stderr gracefully.
