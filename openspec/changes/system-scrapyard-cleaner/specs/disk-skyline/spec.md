# Spec Delta: disk-skyline

## ADDED Requirements

### Requirement: Scrapyard CLI Flag and TUI Mode Transition
The system SHALL support launching directly into the Scrapyard survey via the `--scrapyard` CLI flag, and dynamically toggling between normal directory survey and the virtual Scrapyard Street in the terminal TUI via the `[J]` key.

#### Scenario: Launching with --scrapyard argument
- **WHEN** the user executes `disk-skyline --scrapyard`
- **THEN** the system bypasses filesystem traversal, scans system junk categories, and displays the ASCII Scrapyard Skyline in the terminal

#### Scenario: Toggling Scrapyard mode in TUI
- **WHEN** the user presses `[J]` during active TUI exploration
- **THEN** the TUI replaces the active avenue with the Scrapyard Avenue, updating HUD prompts to reflect `[D] Demolish Target` and `[C] Clean All`
