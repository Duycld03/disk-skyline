# i18n Specification

## Purpose

Provides a bilingual internationalization (i18n) engine supporting Vietnamese and English with automatic OS locale detection, CLI override, and runtime language switching.

## Requirements

### Requirement: Bilingual Language Resolution
The system SHALL support Vietnamese (`vi`) and English (`en`) languages. When initialized without an explicit language option, the system SHALL inspect operating system environment variables (`LC_ALL`, `LC_MESSAGES`, `LANG`) and default to Vietnamese if the detected locale begins with `vi`, defaulting to English otherwise.

#### Scenario: Detecting Vietnamese system locale
- **WHEN** the environment variable `LANG` contains `vi_VN.UTF-8`
- **THEN** the system automatically selects Vietnamese as the active language

#### Scenario: Detecting non-Vietnamese or fallback locale
- **WHEN** the environment variables are unset or set to `en_US.UTF-8` or `C`
- **THEN** the system automatically selects English as the active language

### Requirement: CLI Language Override
The system SHALL accept an optional `--lang` (short `-l`) argument accepting values `vi` or `en`. When supplied, this CLI argument SHALL take precedence over environment variable detection.

#### Scenario: Explicit CLI language selection
- **WHEN** the user executes `disk-skyline --lang vi` on an English operating system
- **THEN** the application initializes and renders all interface elements in Vietnamese

### Requirement: Dynamic Runtime Language Toggle
The system SHALL allow users to toggle the active language dynamically between Vietnamese and English during TUI execution by pressing the `L` key. The system SHALL re-render the HUD, street avenue headers, controls help, and modal dialogues immediately in the new language without losing navigation state or rescanning the filesystem.

#### Scenario: Toggling language during interactive stroll
- **WHEN** the user is viewing the skyline in Vietnamese and presses `L`
- **THEN** the UI switches to English, updating all HUD text, building category descriptions, and controls hints instantly

#### Scenario: Toggling language back
- **WHEN** the user is viewing the skyline in English and presses `L`
- **THEN** the UI switches back to Vietnamese
