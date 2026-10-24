# system-protection Specification

## Purpose

Enforces operating system protection by excluding virtual pseudo-filesystems from scans and disabling demolition features for core system directories and files.

## Requirements

### Requirement: Kernel Virtual Filesystem Exclusion
The system SHALL exclude virtual pseudo-filesystems from recursive traversal during scanning. On Linux systems, the scanner SHALL skip `/proc`, `/sys`, `/dev`, and `/run`. On Windows systems, the scanner SHALL skip NTFS junction points and system volume metadata paths.

#### Scenario: Scanning Linux root directory
- **WHEN** the user initiates a scan on `/` or traverses past root directories
- **THEN** the system bypasses `/proc`, `/sys`, `/dev`, and `/run` without traversing their virtual contents or incorporating virtual memory sizes

#### Scenario: Scanning Windows drive root
- **WHEN** the user scans `C:\` on Windows
- **THEN** the system skips NTFS junction loops such as recursive `Application Data` pointers and `System Volume Information`

### Requirement: System Protection Classification
The system SHALL classify core operating system files and directories with an `is_system_protected` flag. On Linux, this SHALL include `/bin`, `/sbin`, `/usr`, `/etc`, `/boot`, `/root`, and the root directory. On Windows, this SHALL include `Windows`, `Program Files`, `pagefile.sys`, `hiberfil.sys`, and `swapfile.sys`.

#### Scenario: Flagging Linux system directory
- **WHEN** the scanner encounters `/usr` or `/etc`
- **THEN** the resulting node is marked with `is_system_protected = true`

#### Scenario: Flagging Windows kernel files
- **WHEN** the scanner encounters `pagefile.sys` or `hiberfil.sys` on a Windows volume
- **THEN** the resulting node is marked with `is_system_protected = true`

### Requirement: Demolition Lockdown on Protected Buildings
The system SHALL disable demolition operations for any building marked as system-protected. When a user presses `d` or `Delete` while focusing on a protected building, the system SHALL reject the action, display an operational rejection notice, and SHALL NOT display any confirmation dialog.

#### Scenario: Attempting demolition on protected directory
- **WHEN** the user focuses on `/etc` or `/usr` and presses `d` or `Delete`
- **THEN** the system suppresses the demolition modal and displays a warning stating that system infrastructure cannot be demolished

#### Scenario: HUD reflects protected status
- **WHEN** the user focuses on a system-protected building
- **THEN** the HUD inspection pane displays a locked badge and indicates that demolition is disabled
