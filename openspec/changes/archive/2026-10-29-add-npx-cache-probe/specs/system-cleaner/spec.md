# Spec Delta

## MODIFIED Requirements

### Requirement: System Environment Junk Detection and Quantification
The system SHALL execute non-blocking probes to detect and quantify reclaimable storage across Docker, Snap package revisions, global package manager caches, and system archives without deleting active project files.

#### Scenario: Running system junk probe
- **WHEN** the user initiates the Scrapyard survey via hotkey `[J]` or CLI `--scrapyard`
- **THEN** the system gathers itemized byte sizes and cleanable descriptors for all detected categories in under 1 second without stalling the UI

#### Scenario: Handling missing or uninstalled tools
- **WHEN** a probed subsystem (such as Docker or Snap) is not installed or the daemon is unreachable
- **THEN** the system gracefully records zero reclaimable bytes and excludes that subsystem without raising fatal errors

#### Scenario: Probing NPX ephemeral execution cache
- **WHEN** the system surveys package caches and `~/.npm/_npx` exists with total size exceeding 10 MB
- **THEN** the system reports a distinct cleanable target with id `npx` and reclaimable byte count matching the directory contents

### Requirement: Individual Target Purging
The system SHALL allow users to trigger targeted cleanup of an individual surveyed junk category. Upon confirmation, the system SHALL execute the corresponding non-destructive cleanup action and verify the reclaimed disk space.

#### Scenario: Purging Docker dangling artifacts
- **WHEN** the user confirms demolition of the Docker junk target
- **THEN** the system runs `docker system prune -f`, removes dangling images and unused volumes, and emits a localized success banner

#### Scenario: Purging package manager caches
- **WHEN** the user confirms demolition of a package cache target (e.g., npm or pip)
- **THEN** the system empties the corresponding cache directory and reports the reclaimed byte count

#### Scenario: Purging NPX ephemeral cache
- **WHEN** the user triggers demolition of the `npx` cache target
- **THEN** the system deletes all contents within `~/.npm/_npx`, re-creates the empty directory, and reports the reclaimed byte count
