# Spec Delta

## ADDED Requirements

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
