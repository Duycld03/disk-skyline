# Proposal

## Why

Traditional terminal disk usage analyzers (like `ncdu`, `gdu`, or `du`) present storage as hierarchical lists, tree tables, or bar charts. While functional, they lack an intuitive spatial metaphor and make disk exploration feel like spreadsheet auditing. 

`disk-skyline` transforms disk exploration into an interactive cyberpunk/urban skyline in the terminal, rendering directories and files as buildings on a city street. This provides an immediate, engaging visual sense of data scale, file types, and disk bloat while keeping navigation fast, lightweight, and fun.

## What Changes

- **In-Memory Bounded Disk Scanner**: Scans the target directory once upon startup up to an optimized depth (5-6 levels) into an in-memory tree with symlink loop protection and cross-filesystem boundary guards, eliminating continuous disk I/O during navigation.
- **Interactive Skyline TUI Viewport**: Renders current directory entries as a 2D city skyline where building height scales logarithmically with file/folder size, width reflects item footprint, and architectural textures/colors correspond to file categories.
- **Street Navigation & Drill-Down**: Supports horizontal camera panning (`h`/`l` or arrow keys) along the street with a HUD minimap, entering buildings/directories (`Enter`) to explore internal street skylines, and exiting back (`Esc`/`Backspace`) with breadcrumbs.
- **Restricted Area (Bunker) Classification**: Treats hidden files and directories (`.dotfiles`) as military bunkers / restricted zones with warning fences and hazard patterns while preserving accurate disk size accounting.
- **Safe Demolition System**: Provides an in-TUI demolition command (`d` or `Delete`) with two-stage confirmation (simple `y` for normal items, explicit `YES` prompt for restricted or massive items), instantly freeing disk space and pruning the in-memory tree without full disk rescans.
- **On-Demand Refresh**: Supports manual rescan trigger (`r` or `F5`) when external filesystem changes occur.

## Capabilities

### New Capabilities
- `disk-skyline`: Core interactive terminal application providing in-memory disk scanning, 2D city skyline TUI rendering, street drill-down navigation, restricted zone classification for dotfiles, and safe demolition.

### Modified Capabilities
<!-- None: This is a greenfield project with no existing specs. -->

## Impact

- **New Rust Project**: Initializes a Cargo binary crate (`disk-skyline`) with dependencies including `ratatui`, `crossterm`, `rayon`, `jwalk` (or `walkdir`), and `clap`.
- **Performance**: High frame rate (60 FPS) rendering with zero disk I/O during navigation; disk walk happens only once at startup or upon explicit manual refresh.
- **Safety**: Safe deletion guardrails preventing accidental removal of critical files.
