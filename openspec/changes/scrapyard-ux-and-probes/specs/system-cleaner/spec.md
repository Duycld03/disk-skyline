# Spec Delta: system-cleaner

## MODIFIED Requirements

### Requirement: System Environment Junk Detection and Quantification
The system SHALL execute non-blocking probes to detect and quantify reclaimable storage across Docker, Snap package revisions, global package manager caches (including Pip caches >= 1 MB, UV tool and python caches in `~/.local/share/uv`, Cargo, NPM, and NuGet), system archives, and `/tmp` temporary directories without deleting active project files.

#### Scenario: Running system junk probe
- **WHEN** the user initiates the Scrapyard survey via hotkey `[J]` or CLI `--scrapyard`
- **THEN** the system gathers itemized byte sizes and cleanable descriptors for all detected categories in under 1 second without stalling the UI

#### Scenario: Handling missing or uninstalled tools
- **WHEN** a probed subsystem (such as Docker or Snap) is not installed or the daemon is unreachable
- **THEN** the system gracefully records zero reclaimable bytes and excludes that subsystem without raising fatal errors

#### Scenario: Detecting temporary directory buildup
- **WHEN** the user's `/tmp` directory contains user-owned temporary artifacts exceeding 50 MB
- **THEN** the system detects and quantifies the temporary debris as a cleanable target on the Scrapyard Avenue

### Requirement: Bulldozer Batch Cleanup (Clean All)
The system SHALL provide a batch cleanup action (`[C]`) that sequentially executes cleanup across all user-level non-elevated junk categories without triggering administrative elevation prompts or PolicyKit popups, updating the overall disk capacity and presenting a consolidated reclamation report. Targets requiring `sudo` (e.g. Snap revisions and APT archives) SHALL be excluded from unattended batch execution.

#### Scenario: Confirming and executing batch cleanup
- **WHEN** the user confirms the batch cleanup prompt
- **THEN** all cleanable categories are purged sequentially, the aggregate freed space is calculated, and the disk usage indicators are refreshed

#### Scenario: Confirming and executing batch cleanup without sudo prompts
- **WHEN** the user confirms the batch cleanup prompt
- **THEN** all safe user-level categories (Docker, package caches, temporary files) are purged sequentially without triggering PolicyKit or sudo password dialogs, and the reclaimed space is refreshed
