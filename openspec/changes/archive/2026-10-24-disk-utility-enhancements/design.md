# Design

## Context

See `proposal.md` for motivation. Currently, `disk-skyline` provides terminal and 2D GUI navigation, but lacks direct interaction with host applications, rapid jumping across long avenues, and visual clues for identifying abandoned files.

## Goals / Non-Goals

**Goals:**
- Provide instant non-blocking opening of targeted files/folders using the host OS default handler via `o`.
- Implement lightweight real-time Spotlight search (`/`) that immediately jumps camera and cursor to matched buildings.
- Record file modified timestamps during directory scanning to visually highlight stale/abandoned clutter (>180 days).
- Render a 14px street radar minimap bar visualizing all buildings along the current avenue and the avatar's location.

**Non-Goals:**
- In-app file editing or embedded file viewer.
- Heavy audio synthesizer or sound libraries.
- Deep background scanning beyond the current directory depth.

## Decisions

### 1. Zero-Dependency Non-Blocking OS File Opener
Instead of pulling in third-party crates, use standard `std::process::Command` with detached execution:
- Linux: `xdg-open "<path>"`
- macOS: `open "<path>"`
- Windows: `cmd.exe /C start "" "<path>"`

Spawned processes discard standard I/O and do not wait, ensuring the game loop and terminal remain 100% interactive at 60 FPS.

### 2. In-Memory Spotlight Search (`/`)
- A search query buffer is added to `App`: `pub search_query: Option<String>`.
- In TUI and GUI:
  - Pressing `/` activates search mode.
  - Text input appends to `search_query`.
  - The first building whose name contains `search_query` (case-insensitive) is selected.
  - In GUI, the camera lerps toward the matched building.
  - Pressing `Enter` or `Esc` concludes search mode while keeping the selection.

### 3. File Staleness & Visual Age Modeling
- `BuildingNode` records `pub modified_secs_ago: Option<u64>` during scan from `std::fs::metadata(path).modified()`.
- Nodes with `modified_secs_ago >= 180 * 86400` (~6 months) are marked stale:
  - Facade window brightness is modulated (dimmed by 50%).
  - A subtle `[OLD]` badge or muted facade border indicates abandoned storage.

### 4. Street Radar Minimap Bar
- Drawn beneath the top header: width `screen_w - 24.0`, height `14.0`.
- Renders a miniature height profile using each building's scaled vertical height.
- Displays an amber/cyan position indicator representing the avatar's current location relative to total street length.

## Risks / Trade-offs

- **Risk:** Filesystems with unavailable `mtime` (e.g. certain virtual filesystems).
  - *Mitigation:* Fall back to `None` and treat as normal active building.
- **Risk:** Unmatched search queries.
  - *Mitigation:* Display red border around search bar when 0 matches found; do not jump selection until a match is found.
