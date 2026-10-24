# Proposal

## Why

When users run `disk-skyline` at the system root directory (`/` on Linux or `C:\` on Windows), two severe hazards emerge:
1. Virtual pseudo-filesystems (such as `/proc`, `/sys`, `/dev`, and `/run` on Linux) and NTFS junction loops on Windows stall traversal or distort storage statistics (e.g., `/proc/kcore` reporting 128 TB of virtual memory).
2. Users running with elevated privileges (`sudo` or Administrator) could accidentally trigger demolition on vital operating system infrastructure (such as `/etc`, `/usr`, `/boot`, or `C:\Windows`), causing catastrophic OS failure.

This change introduces system root protection: ignoring virtual pseudo-filesystems during scan and disabling demolition completely on core OS infrastructure.

## What Changes

- **Virtual Pseudo-Filesystem Exclusion**: Automatically skips virtual kernel directories (`/proc`, `/sys`, `/dev`, `/run`) on Linux and junction loops / system volume metadata on Windows during traversal, ensuring fast scanning and accurate physical disk accounting.
- **System Protection Classification**: Identifies and flags operating system infrastructure (such as `/bin`, `/sbin`, `/usr`, `/etc`, `/boot`, `/root` on Linux; `C:\Windows`, `C:\Program Files`, `pagefile.sys`, `hiberfil.sys` on Windows; and the root node itself) with `is_system_protected = true`.
- **Demolition Lockdown for Protected Landmarks**: Completely disables file deletion (`d` or `Delete`) for system-protected buildings. Pressing demolition keys on protected buildings immediately displays a rejection notification without presenting any confirmation modal.
- **Visual Shield & Locked HUD Badges**: Displays a protective badge (`[LOCKED]`/`[SYS]`) on building facades and updates the HUD shortcut label to reflect that demolition is disabled for the selected landmark.

## Capabilities

### New Capabilities
- `system-protection`: Operating system protection rules, virtual filesystem exclusion, and demolition lockdown for critical system infrastructure.

### Modified Capabilities
<!-- None: This builds upon the baseline disk-skyline capabilities. -->

## Impact

- **Scanner Performance**: Greatly accelerates scans of root directories by eliminating recursive traversal of kernel devices and process maps.
- **Safety**: Guarantees zero risk of accidental operating system destruction through `disk-skyline` even when running as superuser/root.
