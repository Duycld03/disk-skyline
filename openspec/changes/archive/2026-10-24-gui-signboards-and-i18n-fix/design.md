# Design

## Context

See `proposal.md` for motivation. Currently, the 2D Game GUI renders buildings as anonymous facades where names only become visible when standing directly in front of the building. Furthermore, navigation errors in `src/app.rs` are hardcoded in Vietnamese, causing language desync in English mode, and Macroquad's default ASCII font lacks Vietnamese Unicode glyphs, rendering them as `[][]` boxes.

## Goals / Non-Goals

**Goals:**
- Provide instant visual identification for every building on the street from any distance:
  - Upper facade neon signboard with category tag and truncated file name.
  - Street-level nameplate and formatted size under each building on the sidewalk curb.
- Localize 100% of runtime navigation feedback messages in `src/app.rs` via `src/i18n.rs`.
- Embed and load a clean TrueType font supporting Vietnamese Unicode characters in `src/gui/mod.rs` so text in both languages renders without broken box glyphs (`[][]`).

**Non-Goals:**
- Network font loading or external asset downloading (the font will be embedded directly in the binary at compile time).
- Cluttering the skyline with oversized banners (signboards must scale cleanly within each building's bounding width).

## Decisions

### 1. Dual-Tier Building Identification

**Decision:** Render two complementary identifiers for each building:
1. **Upper Facade Neon Signboard**: Positioned at `top_y + 4.0` to `top_y + 22.0` on the building facade. Contains an abbreviated badge (e.g. `[DIR]`, `[RS]`, `[MD]`, `[ZIP]`, `[BIN]`, `[!]`) and truncated name. Drawn with high-contrast text matching the category color.
2. **Street-Level Nameplate**: Positioned along the curb beneath each building (`ground_y + 2.0` to `ground_y + 14.0`). Displays the filename and formatted size (`format_bytes(node.size)`).
**Rationale:** Matches the clarity of the Terminal TUI view, allowing the player to scan the entire street at a glance without having to walk up to each door.

### 2. Comprehensive Navigation Localization in `src/i18n.rs`

**Decision:** Add dictionary methods to `Language`:
- `drill_down_file_err(&self, name: &str) -> String`
- `drill_down_empty_err(&self) -> &'static str`
- `drill_down_entered(&self, name: &str) -> String`
- `navigate_up_success(&self) -> &'static str`
- `zoning_switched(&self, mode: &str) -> String`
Update `App::drill_down()`, `App::navigate_up()`, and `App::toggle_sort()` in `src/app.rs` to use `self.language`.
**Rationale:** Prevents hardcoded language leaking across sessions, ensuring English mode remains 100% English.

### 3. TrueType Font Embedding via Macroquad

**Decision:** Embed a clean monospace TrueType font supporting Vietnamese diacritics (such as `DejaVuSansMono.ttf`) using `include_bytes!` and `load_ttf_font_from_bytes`. Store the loaded `Font` in `GuiApp` and pass it to drawing routines via `draw_text_ex`.
**Rationale:** Macroquad's default font only covers ASCII 0-127. Embedding a TrueType font resolves all missing glyph box characters (`[][]`) with zero external runtime dependencies.

## Risks / Trade-offs

- **Risk:** Embedded font increases binary size.
  - *Mitigation:* `DejaVuSansMono.ttf` is ~336 KB, which keeps `disk-skyline-gui` under 3 MB, preserving lightweight performance.
- **Risk:** Narrow buildings might clip long filenames.
  - *Mitigation:* Truncate names longer than the building width with ellipsis (`…`) so signs always fit neatly within the building width.
