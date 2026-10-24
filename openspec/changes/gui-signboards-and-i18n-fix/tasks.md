# Tasks

## 1. Navigation i18n Localization

- [x] 1.1 Add localized status message methods (`drill_down_file_err`, `drill_down_empty_err`, `drill_down_entered`, `navigate_up_success`) to `Language` in `src/i18n.rs`.
- [x] 1.2 Refactor `App::drill_down()`, `App::navigate_up()`, and `App::toggle_sort()` in `src/app.rs` to emit localized messages based on `self.language`.
- [x] 1.3 Add unit tests in `src/app.rs` and `src/i18n.rs` verifying feedback strings match the active language in both English and Vietnamese.

## 2. Unicode Font Embedding & Rendering

- [x] 2.1 Add clean TrueType font asset supporting full Vietnamese Unicode diacritics and load it via `macroquad::text::load_ttf_font_from_bytes`.
- [x] 2.2 Integrate the loaded `Font` into `GuiApp` and pass it to all text rendering calls in `src/gui/render.rs` via `draw_text_ex`.
- [x] 2.3 Verify that Vietnamese text renders cleanly without any missing glyph square box characters (`[][]`).

## 3. Building Facade Signboards & Street Nameplates

- [x] 3.1 Implement upper facade neon signboard rendering on every building with category indicator badges (`[DIR]`, `[RS]`, `[ZIP]`, `[BIN]`, `[!]`).
- [x] 3.2 Implement street-level nameplate and size label rendering along the sidewalk curb beneath each building.
- [x] 3.3 Add text truncation logic to keep building names neatly contained within each structure's width.

## 4. Verification & Quality Assurance

- [x] 4.1 Run `cargo test` to verify all 25+ unit and integration tests pass without regression.
- [x] 4.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` to ensure strict code quality.
- [x] 4.3 Run smoke verification on the updated GUI and update the documentation screenshot in `README.md`.
