# Spec Delta

## ADDED Requirements

### Requirement: Localized Navigation Feedback and Font Glyph Integrity
The application SHALL localize all runtime navigation feedback messages through the active language dictionary and ensure graphical window font rendering displays all characters without square box glyphs (`[][]`).

#### Scenario: Attempting to drill down into a single file in English mode
- **WHEN** the application language is set to English and the user attempts to enter a single file
- **THEN** the system displays an English error notice without mixed Vietnamese text

#### Scenario: Rendering Vietnamese diacritics without broken boxes
- **WHEN** the application displays Vietnamese text in the graphical window
- **THEN** accented letters and special characters render cleanly without square box glyphs (`[][]`)
