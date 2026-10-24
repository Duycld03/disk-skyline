# Proposal

## Why

In the 2D Game GUI, buildings currently appear as anonymous structures with no identifying labels unless the avatar walks directly in front of them. Furthermore, attempting to enter a single file displays a hardcoded Vietnamese message even when English is active, and Macroquad's default ASCII font renders Vietnamese diacritics as broken square boxes (`[][]`). Adding visible building signboards, street nameplates, and localized navigation messages with proper Unicode text rendering makes the skyline instantly recognizable from afar and eliminates font desync.

## What Changes

- **Building Signboards & Identification**:
  - Add upper facade neon signboards and rooftop badges (`[DIR] target/`, `[RS] model.rs`, `[CFG] Cargo.toml`, `[!] .git`) to each building so they are clearly identifiable from afar without walking to them.
  - Add street-level nameplates and size labels below the sidewalk curb under each building, matching the clarity of the Terminal TUI.
- **Bilingual Navigation Status Messages**:
  - Move hardcoded status messages in `App::drill_down()`, `App::navigate_up()`, and related navigation actions into the `Language` dictionary in `src/i18n.rs`.
  - Ensure all runtime feedback strictly matches the active language (`Language::En` produces 100% English, `Language::Vi` produces 100% Vietnamese).
- **Unicode Font Rendering**:
  - Load and render full UTF-8 Vietnamese glyphs without missing box characters (`[][]`), ensuring clean, legible text in both languages.

## Capabilities

### Modified Capabilities

- `game-gui`: Add building visual identification (upper facade neon signs and street nameplates visible from afar).
- `i18n`: Add complete navigation status message localization and Unicode UTF-8 font rendering support without unrenderable glyph boxes.

### New Capabilities

None.
