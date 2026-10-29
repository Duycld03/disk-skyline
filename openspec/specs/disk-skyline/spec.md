# disk-skyline Specification

## Purpose

Provides an interactive terminal user interface that scans filesystem storage once into an in-memory tree, visualizes directory contents as an urban skyline with logarithmic building heights, supports drill-down street navigation, marks hidden items as restricted bunkers, and allows safe file demolition.

## Requirements

### Requirement: Bounded In-Memory Disk Scanning
The system SHALL scan the target directory upon launch up to a configurable depth limit (defaulting to 6 levels) into an in-memory node hierarchy. The system SHALL calculate recursive sizes for all directories and SHALL ignore recursive symlink loops and cross-filesystem mount points.

#### Scenario: Successful initial scan
- **WHEN** the user launches the application targeting a directory path
- **THEN** the system displays a loading progress screen, constructs the in-memory tree within the depth boundary, and transitions to the interactive skyline without further background disk I/O

#### Scenario: Symlink loop encountered
- **WHEN** the scanner encounters a symbolic link pointing back to an ancestor directory
- **THEN** the system skips descending into the link and records the link without infinite recursion

### Requirement: 2D Skyline TUI Visualization
The system SHALL render the active directory's child items as a 2D city skyline above a horizontal street line. Building heights SHALL scale logarithmically relative to the maximum item size in the current directory, and building textures and colors SHALL reflect file categories (directories, code, media, archives, binaries, documents).

#### Scenario: Rendering skyline buildings
- **WHEN** the active directory contains files and subdirectories
- **THEN** each item is rendered as a distinct building whose vertical height reflects its logarithmic size bracket and whose facade displays category-specific glyphs and colors

#### Scenario: Extreme size disparity
- **WHEN** a directory contains both a 10 KB file and a 10 GB file
- **THEN** the logarithmic scaling ensures the small file remains visibly rendered with at least minimum foundation height and the large file does not exceed viewport bounds

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
### Requirement: Restricted Area Classification for Hidden Files
The system SHALL classify any file or directory whose name starts with a dot (`.`) as a restricted zone. The system SHALL calculate its disk footprint normally while rendering it on the skyline with distinctive bunker/hazard motifs and a restricted warning tag.

#### Scenario: Rendering a hidden directory
- **WHEN** the current directory contains a hidden directory such as `.git`
- **THEN** the system renders the item as a low-profile bunker with hazard or barbed wire patterns and displays a restricted badge in the HUD inspection pane

### Requirement: Safe Demolition with Two-Tier Confirmation
The system SHALL provide a demolition command triggered by `d`, `x`, or `Delete`. The system SHALL open a modal confirmation dialog detailing the item path and reclaimed space. The system SHALL require `y` for standard items and require typing `YES` for restricted items or items exceeding 1 GB before executing filesystem deletion.

#### Scenario: Confirming standard file demolition
- **WHEN** the user presses `d` on a standard file under 1 GB and confirms with `y`
- **THEN** the system deletes the file from disk, removes the building from the in-memory tree, updates the parent directory size, and updates the skyline view without a full rescan

#### Scenario: Confirming standard file demolition with x key
- **WHEN** the user presses `x` on a standard file under 1 GB and confirms with `y`
- **THEN** the system deletes the file from disk, removes the building from the in-memory tree, updates the parent directory size, and updates the skyline view without a full rescan

#### Scenario: Confirming restricted item demolition
- **WHEN** the user presses `d` on a restricted bunker item or item exceeding 1 GB
- **THEN** the system requires the user to type `YES` before deleting the target from disk

#### Scenario: Cancelling demolition
- **WHEN** the user presses `Esc` or `n` within the demolition modal
- **THEN** the modal closes and no filesystem changes occur

### Requirement: On-Demand Skyline Rescan
The system SHALL allow the user to trigger a manual filesystem rescan by pressing `r` or `F5`. The system SHALL re-walk the current directory tree from disk, update the in-memory nodes, and refresh the skyline display.

#### Scenario: Triggering manual rescan
- **WHEN** the user presses `r` during navigation
- **THEN** the system re-scans the active subtree from disk, synchronizes the in-memory tree, and updates the viewport

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

### Requirement: Scrapyard CLI Flag and TUI Mode Transition
The system SHALL support launching directly into the Scrapyard survey via the `--scrapyard` CLI flag, and dynamically toggling between normal directory survey and the virtual Scrapyard Street in the terminal TUI via the `[J]` key.

#### Scenario: Launching with --scrapyard argument
- **WHEN** the user executes `disk-skyline --scrapyard`
- **THEN** the system bypasses filesystem traversal, scans system junk categories, and displays the ASCII Scrapyard Skyline in the terminal

#### Scenario: Toggling Scrapyard mode in TUI
- **WHEN** the user presses `[J]` during active TUI exploration
- **THEN** the TUI replaces the active avenue with the Scrapyard Avenue, updating HUD prompts to reflect `[D] Demolish Target` and `[C] Clean All`
