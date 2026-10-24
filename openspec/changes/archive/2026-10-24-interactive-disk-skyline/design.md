# Design

## Context

`disk-skyline` is a greenfield Rust console application. The core architectural challenge is balancing deep filesystem exploration with instant, smooth (60 FPS) terminal rendering, while avoiding repeated disk I/O and accidental data loss during file cleanup. See `proposal.md` for motivation and `specs/disk-skyline/spec.md` for behavioral requirements.

## Goals / Non-Goals

**Goals:**
- Construct an in-memory directory tree once at startup with bounded recursive size calculations.
- Render an interactive 2D skyline in the terminal using `ratatui` and `crossterm` with horizontal panning and minimap.
- Provide smooth drill-down street navigation (Enter/Esc) with zero filesystem latency.
- Stylize dotfiles/hidden items as military bunkers / restricted areas.
- Implement safe demolition with two-tier confirmation modals and live in-memory updates.
- Provide on-demand manual rescan (`r`).

**Non-Goals:**
- Continuous background filesystem watching (e.g., `notify` / `inotify`) or auto-syncing with external changes.
- In-app file editing, text editing, or file compression utilities.
- Cross-device or remote filesystem protocol support (e.g., SSH/S3).
- Graphical (GUI) or browser-based rendering.

## Decisions

### 1. Technology Stack: Rust, Ratatui, and Crossterm
- **Decision**: Build the application in Rust using `ratatui` (v0.28+) for declarative terminal rendering, `crossterm` for terminal event handling, `clap` (derive) for CLI argument parsing, and `rayon` / `walkdir` for fast filesystem traversal.
- **Rationale**: Rust produces a single self-contained binary with zero runtime dependencies. `ratatui` provides an immediate-mode TUI framework with high rendering efficiency, avoiding screen flicker through differential terminal buffer diffs.
- **Alternatives considered**:
  - *Go + Bubbletea*: Excellent TUI aesthetics, but Rust provides tighter control over memory layouts and faster parallel filesystem traversal without garbage collection pauses.
  - *Python + Textual*: Faster to prototype, but slower startup and traversal speeds for large directories, requiring an installed Python runtime.

### 2. In-Memory Tree Architecture (`BuildingNode`)
- **Decision**: Represent filesystem state in an in-memory recursive data structure:
  ```rust
  pub struct BuildingNode {
      pub name: String,
      pub path: PathBuf,
      pub is_dir: bool,
      pub is_hidden: bool,
      pub size: u64,
      pub file_type: BuildingCategory,
      pub children: Vec<BuildingNode>,
  }
  ```
- **Rationale**: Scanning the tree once at startup allows instantaneous navigation (O(1) memory lookup) when switching streets or drilling into directories.
- **Traversal Constraints**:
  - Bounded initial depth: Default recursion depth set to 6 levels to prevent scanning stalls on deeply nested structures like monorepos.
  - Filesystem boundary guard: Do not traverse across filesystem boundaries or follow symlink loops (`follow_links(false)`).

### 3. Logarithmic Skyline Projection Engine
- **Decision**: Map item size to building height using a logarithmic scale:
  $$\text{height} = \text{MIN\_HEIGHT} + \left\lfloor \frac{\ln(1 + \text{size})}{\ln(1 + \text{max\_size})} \times (\text{MAX\_HEIGHT} - \text{MIN\_HEIGHT}) \right\rfloor$$
- **Rationale**: File sizes on disk frequently span 8 orders of magnitude (from a few bytes to hundreds of gigabytes). Linear scaling renders small files invisible or large files off-screen. Logarithmic scaling guarantees all files have a visible foundation while preserving relative scale.
- **Architectural Theming**:
  - `Directory`: Commercial office towers with window arrays `[::]`, cyan highlights.
  - `Code / Source`: High-tech towers with matrix glyphs `<>`, green accents.
  - `Media`: Neon entertainment centers with blocks `[][]`, magenta/purple styling.
  - `Archives / Binaries`: Industrial warehouses `#`, amber/red accents.
  - `Restricted / Dotfiles`: Low-profile concrete bunkers with barbed wire `xxxx` and hazard badges `[!] RESTRICTED`.

### 4. Viewport and Camera Navigation State
- **Decision**: Maintain a horizontal street coordinate system:
  ```rust
  pub struct NavigationState {
      pub current_path_indices: Vec<usize>, // Breadcrumb index stack
      pub selected_index: usize,           // Focused building on active street
      pub viewport_offset_x: u16,          // Horizontal scroll offset in terminal cols
      pub sort_mode: SortMode,             // Size, Name, Category
  }
  ```
- **Rationale**: Terminal width is bounded (80-160 columns), whereas a directory may contain hundreds of items. Horizontal scrolling shifts the street camera while keeping the selected building visible and centered.

### 5. Safe Demolition and Tree Mutation
- **Decision**: Implement demolition through a modal overlay:
  1. User triggers `d` or `Delete`.
  2. Modal inspects target: If target is marked `is_hidden` or `size > 1 GB`, require user to type `YES`. Otherwise, require single key `y`.
  3. Execute deletion via `std::fs::remove_dir_all` or `std::fs::remove_file`.
  4. On success, remove the node from the active parent's `children` vector, propagate the subtracted byte count upwards through ancestor nodes, and close the modal.
- **Rationale**: Immediate in-memory tree mutation avoids an expensive full-disk rescan, providing instantaneous UI feedback while keeping disk numbers consistent.

## Risks / Trade-offs

- **[Risk: Huge directories with 500k+ files cause long startup freeze]**
  $\rightarrow$ *Mitigation*: Show an architect blueprint loading screen during initial scan; enforce bounded depth (default 6) with CLI flag `--depth` to override.
- **[Risk: Accidental deletion of critical system files or repository metadata]**
  $\rightarrow$ *Mitigation*: Two-tier confirmation requiring explicit `YES` typing for dotfiles (`.git`, `.env`) and large directories, plus cancel on `Esc`.
- **[Risk: Terminal resize during navigation breaks layout]**
  $\rightarrow$ *Mitigation*: Ratatui redraws dynamically on terminal resize events; calculate building heights and street viewport dynamically based on `terminal_area.height` and `terminal_area.width`.
- **[Risk: In-memory tree becomes stale if files change externally]**
  $\rightarrow$ *Mitigation*: Provide explicit manual rescan (`r` or `F5`) to re-sync the active directory subtree without restarting the app.
