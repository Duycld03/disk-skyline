# Spec Delta

## MODIFIED Requirements

### Requirement: Interactive Street Navigation and Drill-Down
The system SHALL allow users to navigate horizontally along the street using arrow keys (`Left`/`Right`) or vim keys (`h`/`l`), highlighting the focused building and displaying its metadata in a HUD pane. The system SHALL navigate into a directory when `Enter` is pressed, return to the parent directory when `Esc` or `Backspace` is pressed, and toggle display language when `L` is pressed.

#### Scenario: Panning along the street
- **WHEN** the user presses `Right` or `l`
- **THEN** the focus shifts to the adjacent building, scrolling the street viewport horizontally if the building is outside the current screen window

#### Scenario: Entering a directory building
- **WHEN** the user selects a directory building and presses `Enter`
- **THEN** the viewport transitions to render the street skyline corresponding to that directory's contents, updating the breadcrumb trail

#### Scenario: Exiting to parent street
- **WHEN** the user presses `Esc` or `Backspace` while inside a child directory
- **THEN** the viewport returns to the parent street and restores focus to that directory's building

#### Scenario: Toggling UI language hotkey
- **WHEN** the user presses `L` during street navigation
- **THEN** the system alternates the active locale and re-renders the skyline HUD and controls in the newly selected language
