# Spec Delta

## Purpose

Provides a standalone native 2D desktop game window powered by Macroquad, rendering filesystem storage as a pixel-art urban skyline with an animated walking avatar, retro RPG HUD, and real-time interactive navigation.

## ADDED Requirements

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
