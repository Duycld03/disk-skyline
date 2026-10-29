# Spec Delta: disk-skyline

## MODIFIED Requirements

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
