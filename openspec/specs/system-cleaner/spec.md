# system-cleaner Specification

## Purpose
Enables comprehensive detection, quantification, presentation, and safe execution of system-wide developer environment cleanup across Docker, Snap revisions, language package caches, and system archives.

## Requirements

### Requirement: System Environment Junk Detection and Quantification
The system SHALL execute non-blocking probes to detect and quantify reclaimable storage across Docker, Snap package revisions, global package manager caches, and system archives without deleting active project files.

#### Scenario: Running system junk probe
- **WHEN** the user initiates the Scrapyard survey via hotkey `[J]` or CLI `--scrapyard`
- **THEN** the system gathers itemized byte sizes and cleanable descriptors for all detected categories in under 1 second without stalling the UI

#### Scenario: Handling missing or uninstalled tools
- **WHEN** a probed subsystem (such as Docker or Snap) is not installed or the daemon is unreachable
- **THEN** the system gracefully records zero reclaimable bytes and excludes that subsystem without raising fatal errors

### Requirement: Individual Target Purging
The system SHALL allow users to trigger targeted cleanup of an individual surveyed junk category. Upon confirmation, the system SHALL execute the corresponding non-destructive cleanup action and verify the reclaimed disk space.

#### Scenario: Purging Docker dangling artifacts
- **WHEN** the user confirms demolition of the Docker junk target
- **THEN** the system runs `docker system prune -f`, removes dangling images and unused volumes, and emits a localized success banner

#### Scenario: Purging package manager caches
- **WHEN** the user confirms demolition of a package cache target (e.g., npm or pip)
- **THEN** the system empties the corresponding cache directory and reports the reclaimed byte count

### Requirement: Bulldozer Batch Cleanup (Clean All)
The system SHALL provide a batch cleanup action (`[C]`) that sequentially executes cleanup across all surveyed junk categories, updating the overall disk capacity and presenting a consolidated reclamation report.

#### Scenario: Confirming and executing batch cleanup
- **WHEN** the user confirms the batch cleanup prompt
- **THEN** all cleanable categories are purged sequentially, the aggregate freed space is calculated, and the disk usage indicators are refreshed
