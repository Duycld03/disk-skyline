# Spec Delta: game-gui

## ADDED Requirements

### Requirement: Virtual Scrapyard Avenue Skyline Rendering
The 2D GUI SHALL support rendering a dedicated "Virtual Scrapyard Avenue" (Đại Lộ Phế Liệu) representing surveyed system junk items as explorable skyline buildings.
Each junk category SHALL be rendered with industrial polluted aesthetic styling, logarithmic vertical heights corresponding to reclaimable bytes, and neon signboards displaying category badges (`[SCRAP]`, `[DOCKER]`, `[SNAP]`, `[CACHE]`).

#### Scenario: Entering the Scrapyard Avenue
- **WHEN** the user presses `[J]` during GUI exploration
- **THEN** the active avenue transitions to the virtual Scrapyard Street displaying all detected junk buildings, and the avatar is placed at the entrance

#### Scenario: Exiting the Scrapyard Avenue
- **WHEN** the user presses `[Esc]` or `[Backspace]` while on the Scrapyard Avenue
- **THEN** the camera and avatar transition back to the previous filesystem directory street

### Requirement: Scrapyard Demolition and Bulldozer Effects
The 2D GUI SHALL support both single-building demolition (`[D]`) and all-building batch demolition (`[C]`). Demolishing a junk building SHALL trigger particle debris collapse animations, remove the building from the street, and emit a localized status banner indicating the reclaimed gigabytes.

#### Scenario: Demolishing a single junk building
- **WHEN** the user presses `[D]` in front of a junk building and confirms `[Y]`
- **THEN** the building collapses into particle explosion debris, the underlying cleanup command is executed, and the building is pruned from the street

#### Scenario: Triggering bulldozer batch wipe
- **WHEN** the user triggers batch cleanup via `[C]` and confirms the prompt
- **THEN** all junk buildings trigger chain explosion particle animations sequentially, and the street clears to an empty avenue with a consolidated success banner
