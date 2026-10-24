# Proposal

## Why

While `disk-skyline` provides an engaging 2D skyline visualization of disk storage, users need practical power tools to efficiently triage and inspect files without leaving the application or getting bogged down by scrolling through long streets. Adding instant external file opening (`o`), spotlight search navigation (`/`), visual file age/staleness indicators, and a street radar minimap turns disk-skyline into a sharp, highly productive disk maintenance station with zero bloat.

## What Changes

- **External File & Folder Opening (`o`)**: Add ability to launch the currently selected file or directory with the operating system's default handler (`open::that_detached` or non-blocking system launcher) in both terminal TUI and 2D Game GUI.
- **Spotlight Search & Jump (`/`)**: Add instant fuzzy/prefix search overlay in both TUI and GUI to filter children by name and immediately pan camera and avatar directly to the matched building.
- **Building Age & Staleness Indicator**: Track last modified time (`mtime`) during directory scans and visually differentiate fresh, recently active files (bright neon lights) from stale/abandoned files older than 6 months or 1 year (dimmed facade, muted windows, `[OLD]` badge) to help users instantly spot clutter.
- **Street Radar Minimap**: Add a lightweight horizontal street radar bar below the header displaying the height profile of all buildings across the current avenue with an active position marker indicating avatar and viewport location.

## Capabilities

### Modified Capabilities

- `disk-skyline`: Add external file opening, spotlight search navigation, and file modified-time tracking to core navigation model and terminal TUI.
- `game-gui`: Add interactive spotlight search prompt overlay, street radar minimap bar, and building age lighting variation.
