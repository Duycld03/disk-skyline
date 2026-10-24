# Spec Delta

## ADDED Requirements

### Requirement: External File Launching
The system SHALL allow users to open the currently selected file or directory in the host operating system's default application by pressing `o`. The launch SHALL execute asynchronously without freezing or blocking application input.

#### Scenario: Opening file with default viewer
- **WHEN** the user selects a file building and presses `o`
- **THEN** the system launches the file with the OS default viewer and displays a non-error status notification

#### Scenario: Opening protected directory
- **WHEN** the user selects a directory and presses `o`
- **THEN** the system opens the directory in the default system file manager

### Requirement: Spotlight Search Navigation
The system SHALL provide an interactive search mode triggered by `/` that filters buildings by name substring or fuzzy matching, instantly moving selection to the best matching building.

#### Scenario: Searching and jumping to building
- **WHEN** the user presses `/` and types a query
- **THEN** the system updates candidate matches in real-time and jumps the selected index to the top candidate

#### Scenario: Cancelling search
- **WHEN** the user presses `Esc` while search mode is active
- **THEN** search input closes and original selection is preserved

### Requirement: File Modification Time Tracking
The directory scanner SHALL record file modification timestamps for each scanned node, categorizing items older than 180 days as stale.

#### Scenario: Scanning file with modification timestamp
- **WHEN** the scanner inspects a filesystem path
- **THEN** it records the file modification time in seconds or days elapsed
