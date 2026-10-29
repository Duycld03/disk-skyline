# Spec Delta

## ADDED Requirements

### Requirement: In-App Privilege Elevation Authentication
The system SHALL provide an interactive In-App Sudo Password Modal in both TUI and GUI modes to authenticate elevated demolition operations without redirecting the user to an external terminal command.

#### Scenario: Prompting for sudo password
- **WHEN** the user initiates demolition on a target marked with `requires_sudo = true`
- **THEN** the system displays an In-App password prompt with masked input (`*`) and execution warnings

#### Scenario: Successful elevated execution
- **WHEN** the user inputs a valid sudo password and confirms the modal
- **THEN** the system passes the password via standard input to `sudo -S`, executes the privileged purge, updates reclaimed space, and clears the password from memory

#### Scenario: Authentication failure or cancellation
- **WHEN** the user enters an incorrect password or presses Escape
- **THEN** the system displays an authentication error banner, preserves the target building on the avenue, and terminates the sudo process safely

## MODIFIED Requirements

### Requirement: System Environment Junk Detection and Quantification
The system SHALL execute non-blocking probes to detect and quantify reclaimable storage across Docker, Snap package revisions, Snap obsolete user profiles, systemd journal logs, developer runtimes, global package manager caches, and system archives without deleting active project files.

#### Scenario: Running system junk probe
- **WHEN** the user initiates the Scrapyard survey via hotkey `[J]` or CLI `--scrapyard`
- **THEN** the system gathers itemized byte sizes and cleanable descriptors for all detected categories in under 1 second without stalling the UI

#### Scenario: Handling missing or uninstalled tools
- **WHEN** a probed subsystem (such as Docker or Snap) is not installed or the daemon is unreachable
- **THEN** the system gracefully records zero reclaimable bytes and excludes that subsystem without raising fatal errors

#### Scenario: Probing extended developer runtimes and logs
- **WHEN** the survey scans user and system directories
- **THEN** it quantifies obsolete Snap profiles (`~/snap/<app>/<old_rev>`), systemd journal logs (`/var/log/journal`), Cockpit update archives, Codex runtimes, and Gradle wrappers

### Requirement: Individual Target Purging
The system SHALL allow users to trigger targeted cleanup of an individual surveyed junk category. Upon confirmation, the system SHALL execute the corresponding non-destructive cleanup action, requesting elevation if required, and verify the reclaimed disk space.

#### Scenario: Purging Docker dangling artifacts
- **WHEN** the user confirms demolition of the Docker junk target
- **THEN** the system runs `docker system prune -f`, removes dangling images and unused volumes, and emits a localized success banner

#### Scenario: Purging package manager caches
- **WHEN** the user confirms demolition of a package cache target (e.g., npm or pip)
- **THEN** the system empties the corresponding cache directory and reports the reclaimed byte count

#### Scenario: Purging elevated targets
- **WHEN** the user confirms demolition of a target requiring elevated privileges (e.g. Snap inactive revisions or Apt archives)
- **THEN** the system triggers the In-App Sudo Password Modal and executes the purge using superuser permissions upon successful authentication

### Requirement: Bulldozer Batch Cleanup (Clean All)
The system SHALL provide a batch cleanup action (`[C]`) that sequentially executes cleanup across surveyed junk categories, updating the overall disk capacity and presenting a consolidated reclamation report.

#### Scenario: Confirming and executing batch cleanup
- **WHEN** the user confirms the batch cleanup prompt
- **THEN** all cleanable categories are purged sequentially, the aggregate freed space is calculated, and the disk usage indicators are refreshed

#### Scenario: Bulldozer with elevated targets
- **WHEN** surveyed junk contains targets requiring sudo and the user initiates Bulldozer cleanup
- **THEN** the system requests sudo authentication once; if authenticated, it purges all targets; if cancelled, it continues purging only user-level targets
