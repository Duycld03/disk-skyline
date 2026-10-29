# Spec Delta: game-gui

## MODIFIED Requirements

### Requirement: Interactive Exploration and Demolition in GUI
The system SHALL support drill-down navigation: pressing `Enter` transitions into the targeted directory's street, and pressing `Backspace` or `Esc` returns to the parent avenue. Directional keys (`Left`/`Right` or `A`/`D`) SHALL be reserved strictly for horizontal walking movement. Pressing `x` or `Delete` SHALL open a game-styled confirmation prompt and, upon confirmation, execute deletion with a pixel particle collapse effect. Pressing `L` SHALL toggle the active language between Vietnamese and English.

#### Scenario: Entering directory building in GUI
- **WHEN** the avatar is positioned at a directory and the user presses `Enter`
- **THEN** the screen transitions to the new street view for that directory

#### Scenario: Demolishing target with particle effect
- **WHEN** the user presses `x` or `Delete` and confirms demolition on an unprotected file
- **THEN** the file is deleted from disk and the building crumbles in a pixel debris particle animation

#### Scenario: Walking right without triggering demolition
- **WHEN** the user holds or presses `D` during exploration
- **THEN** the avatar walks rightward along the sidewalk and no demolition prompt is triggered

### Requirement: Scrapyard Demolition and Bulldozer Effects
The 2D GUI SHALL support both single-building demolition (`[X]` or `[Delete]`) and all-building batch demolition (`[C]`). Demolishing a junk building SHALL trigger particle debris collapse animations, remove the building from the street, and emit a localized status banner indicating the reclaimed gigabytes.

#### Scenario: Demolishing a single junk building
- **WHEN** the user presses `[X]` or `[Delete]` in front of a junk building and confirms `[Y]`
- **THEN** the building collapses into particle explosion debris, the underlying cleanup command is executed, and the building is pruned from the street

#### Scenario: Triggering bulldozer batch wipe
- **WHEN** the user triggers batch cleanup via `[C]` and confirms the prompt
- **THEN** all safe user-level junk buildings trigger chain explosion particle animations sequentially, and the street clears with a consolidated success banner
