# Spec Delta

## MODIFIED Requirements

### Requirement: System Protection Classification
The system SHALL classify core operating system files and directories with an `is_system_protected` flag. On Linux, this SHALL include `/bin`, `/sbin`, `/usr`, `/etc`, `/boot`, `/root`, and the root directory. On Windows, this SHALL include `Windows`, `Program Files`, `pagefile.sys`, `hiberfil.sys`, and `swapfile.sys`. Cleanable Scrapyard junk targets requiring elevation SHALL NOT be classified as system-protected.

#### Scenario: Flagging Linux system directory
- **WHEN** the scanner encounters `/usr` or `/etc`
- **THEN** the resulting node is marked with `is_system_protected = true`

#### Scenario: Flagging Windows kernel files
- **WHEN** the scanner encounters `pagefile.sys` or `hiberfil.sys` on a Windows volume
- **THEN** the resulting node is marked with `is_system_protected = true`

#### Scenario: Cleanable elevated targets are not protected
- **WHEN** the Scrapyard survey identifies a target requiring superuser privileges (e.g. Snap inactive revisions or Apt archives)
- **THEN** the resulting building node has `is_system_protected = false` and `requires_sudo = true`

### Requirement: Demolition Lockdown on Protected Buildings
The system SHALL disable demolition operations for any building marked as system-protected. When a user presses `d` or `Delete` while focusing on a protected building, the system SHALL reject the action, display an operational rejection notice, and SHALL NOT display any confirmation dialog.

#### Scenario: Attempting demolition on protected directory
- **WHEN** the user focuses on `/etc` or `/usr` and presses `d` or `Delete`
- **THEN** the system suppresses the demolition modal and displays a warning stating that system infrastructure cannot be demolished

#### Scenario: HUD reflects protected status
- **WHEN** the user focuses on a system-protected building
- **THEN** the HUD inspection pane displays a locked badge and indicates that demolition is disabled

#### Scenario: Elevated cleanable targets bypass lockdown
- **WHEN** the user focuses on an elevated Scrapyard target with `requires_sudo = true` and `is_system_protected = false`
- **THEN** the system allows demolition by routing to the In-App Sudo Password Modal instead of triggering a lockdown rejection notice
