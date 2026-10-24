# game-gui Specification

## Purpose

Provides a standalone native 2D desktop game window powered by Macroquad, rendering filesystem storage as a pixel-art urban skyline with an animated walking avatar, retro RPG HUD, and real-time interactive navigation.

## Requirements

### Requirement: Native 2D Game Window Initialization
The system SHALL provide a dedicated binary (`disk-skyline-gui`) that opens a native cross-platform graphical window (1280x720 base resolution, resizable) running a 60 FPS game loop. The window SHALL support toggling fullscreen with `F11` and graceful termination via `q`, `Esc` at the root directory, or the window close button.

#### Scenario: Launching the game window
- **WHEN** the user executes `disk-skyline-gui [path]`
- **THEN** the system initializes the window, loads the in-memory filesystem tree, and renders the initial 2D pixel skyline

#### Scenario: Fullscreen toggle
- **WHEN** the user presses `F11`
- **THEN** the game window toggles between windowed and fullscreen mode without reloading data

### Requirement: 2D Pixel Walking Avatar
The system SHALL render an animated 2D pixel character walking on the street sidewalk. When the user holds or presses directional keys (`Left`/`Right` or `A`/`D`), the avatar SHALL smoothly turn toward the movement direction and walk along the street. When the user presses `Space`, the avatar SHALL execute a vertical jump.

#### Scenario: Moving avatar along avenue
- **WHEN** the user presses `Right` or `D`
- **THEN** the avatar turns right, walks toward the right building, and the camera pans smoothly to follow

#### Scenario: Avatar jumping
- **WHEN** the user presses `Space` while grounded on the street
- **THEN** the avatar jumps upward with arcade physics and lands back on the sidewalk

### Requirement: 2D Graphical Pixel Skyline Rendering
The system SHALL render buildings as 2D pixel structures with logarithmic vertical heights. Each building facade SHALL display pixel textures and colors corresponding to its category (Offices, Tech Centers, Media, Archives, Binaries, Documents, and Restricted Bunkers). The background SHALL render a starry night sky with twinkling pixel stars.

#### Scenario: Rendering pixel buildings
- **WHEN** a directory contains items of various types
- **THEN** each item is drawn as a distinct pixel-art building with roofs, windows, doors, and a baseline street

#### Scenario: System protected buildings
- **WHEN** a building is flagged as system-protected
- **THEN** the facade displays a locked padlock icon and red protective warning barrier

### Requirement: Retro Game HUD and Resource Stats
The system SHALL render an arcade-style HUD overlay displaying:
1. Disk free space formatted as a retro Health/Capacity bar (`[████████░░░░]`).
2. Current targeted building stats card (Name, Category, Size, Percent of avenue, Scale).
3. Action command bar with hotkey bindings.

#### Scenario: Viewing target building stats
- **WHEN** the avatar stands in front of a building
- **THEN** the HUD highlights that building's attributes in the RPG stats panel

### Requirement: Interactive Exploration and Demolition in GUI
The system SHALL support drill-down navigation: pressing `Enter` transitions into the targeted directory's street, and pressing `Backspace` or `Esc` returns to the parent avenue. Pressing `d` SHALL open a game-styled confirmation prompt and, upon confirmation, execute deletion with a pixel particle collapse effect. Pressing `L` SHALL toggle the active language between Vietnamese and English.

#### Scenario: Entering directory building in GUI
- **WHEN** the avatar is positioned at a directory and the user presses `Enter`
- **THEN** the screen transitions to the new street view for that directory

#### Scenario: Demolishing target with particle effect
- **WHEN** the user confirms demolition on an unprotected file
- **THEN** the file is deleted from disk and the building crumbles in a pixel debris particle animation

### Requirement: Distant Building Identification and Signboards
The 2D GUI SHALL display persistent identifying markers for every visible building from afar: an upper facade neon signboard showing the category badge and filename, and a street-level nameplate below the sidewalk curb showing the filename and formatted size.

#### Scenario: Identifying unselected buildings across the street
- **WHEN** the user explores an avenue containing multiple buildings and stands in front of one building
- **THEN** all other visible buildings continue to display their neon facade signs and street-level nameplates with sizes, allowing instant identification from afar

#### Scenario: Visual styling of building signs
- **WHEN** a building is rendered in the 2D game window
- **THEN** its upper facade sign is drawn with high-contrast text matching the building's category color theme, and its street nameplate is positioned cleanly below the building base

### Requirement: Street Radar Minimap Bar
The 2D GUI SHALL render a compact horizontal street radar bar below the top header that displays a scaled height profile of all buildings on the current street and marks the avatar's real-time position.

#### Scenario: Visualizing street layout on radar
- **WHEN** the user explores an avenue with varying building heights
- **THEN** the radar bar displays proportional height ticks for each building and a prominent cursor indicator at the avatar's relative coordinate

### Requirement: Spotlight Search Overlay and Camera Jump
The 2D GUI SHALL display an interactive search input overlay when `/` is pressed, dynamically matching filenames and smoothly panning the camera and avatar to the matched building.

#### Scenario: Dynamic camera pan during search
- **WHEN** the user types characters into the search bar
- **THEN** the camera and avatar smoothly animate to focus on the first matching building along the avenue

### Requirement: Building Staleness Visual Aesthetics
The 2D GUI SHALL visually differentiate stale structures (older than 180 days) from active structures by dimming facade and window lighting and displaying an `[OLD]` badge.

#### Scenario: Visual rendering of stale structures
- **WHEN** a building's file has not been modified for more than 180 days
- **THEN** its window brightness is reduced and an `[OLD]` indicator is drawn on its facade

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
