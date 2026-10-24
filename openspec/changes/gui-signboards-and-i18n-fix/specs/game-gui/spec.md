# Spec Delta

## ADDED Requirements

### Requirement: Distant Building Identification and Signboards
The 2D GUI SHALL display persistent identifying markers for every visible building from afar: an upper facade neon signboard showing the category badge and filename, and a street-level nameplate below the sidewalk curb showing the filename and formatted size.

#### Scenario: Identifying unselected buildings across the street
- **WHEN** the user explores an avenue containing multiple buildings and stands in front of one building
- **THEN** all other visible buildings continue to display their neon facade signs and street-level nameplates with sizes, allowing instant identification from afar

#### Scenario: Visual styling of building signs
- **WHEN** a building is rendered in the 2D game window
- **THEN** its upper facade sign is drawn with high-contrast text matching the building's category color theme, and its street nameplate is positioned cleanly below the building base
