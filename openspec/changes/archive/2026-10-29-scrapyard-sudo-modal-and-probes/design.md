# Design

## Context

See `proposal.md` for user motivation and problem context.

Disk Skyline operates across two parallel frontends:
1. **Ratatui Terminal TUI** (`src/app.rs`, `src/ui.rs`, `src/demolish.rs`)
2. **Macroquad 2D Canvas GUI** (`src/gui/mod.rs`, `src/gui/render/overlay.rs`)

Previously, `src/model.rs` mapped `is_system_protected = target.requires_sudo`. Because of this conflation, Scrapyard targets requiring elevated privileges (such as inactive Snap package revisions or APT package archives) were treated as untouchable core OS infrastructure. When users attempted demolition via `d`, `x`, or `Delete`, both frontends completely blocked demolition and presented a passive warning instructing the user to run shell commands manually.

## Goals / Non-Goals

**Goals:**
- Implement five additional high-yield system clutter probes in `src/cleaner/mod.rs` (Snap obsolete user profiles, systemd journal archives, Cockpit update debris, Codex runtime cache, and Gradle wrapper distributions).
- Decouple `is_system_protected` from `requires_sudo` in `BuildingNode` so elevated targets are cleanable.
- Implement an In-App Sudo Password Modal in both Ratatui TUI and Macroquad GUI with masked input (`*`) and retro styling.
- Stream passwords securely to `sudo -S` via child process standard input, zeroing passwords from memory immediately after execution.
- Update Bulldozer (`[C]`) to offer one-time elevation for all privileged items, falling back to user-level cleanup if canceled.

**Non-Goals:**
- Windows UAC integration (this elevation design targets Linux `sudo` environments).
- Hardcoding or storing user passwords to disk or long-lived variables.
- Modifying system files outside the surveyed Scrapyard targets.

## Decisions

### Decision 1: In-App Password Modal over External Polkit / Zenity
- **Choice**: Native in-app modal rendered in Ratatui (TUI) and Macroquad (GUI).
- **Rationale**: Keeps the player immersed in Disk Skyline's pixel-art retro environment. Guarantees consistent operation across graphical desktops, minimal window managers, and SSH / terminal-only sessions where Polkit graphical agents or `zenity` are absent.
- **Alternative Considered**: Invoking `pkexec` or `zenity --password`. Rejected because it introduces external desktop dependencies, breaks headless/terminal workflows, and interrupts the cohesive retro visual design.

### Decision 2: Process Elevation via `sudo -S` over Piped Stdin
- **Choice**: Spawn child processes using `Command::new("sudo").args(["-S", ...]).stdin(Stdio::piped())`.
- **Rationale**: `sudo -S` explicitly instructs `sudo` to read the password from standard input rather than attempting to open `/dev/tty`. This allows Disk Skyline to write the password, flush, close the pipe, and capture execution status and stderr reliably.
- **Alternative Considered**: Pseudo-terminal (PTY) emulation. PTY allocation requires heavy OS-specific crates (`portable-pty` / `nix`) and introduces complex signal handling for what is a standard batch command execution.

### Decision 3: Separation of Protection and Elevation on Data Models
- **Choice**: Add an explicit `pub requires_sudo: bool` field to `BuildingNode` and keep `is_system_protected: false` for all cleanable junk targets.
- **Rationale**: `is_system_protected` is strictly reserved for non-deletable OS root paths (`/bin`, `/usr`, `/etc`). Elevated targets display an elevation badge (⚡ / 🛡️) in the HUD instead of the locked padlock (🔒), signaling to the user that demolition is allowed upon administrative authentication.

### Decision 4: Bulldozer (`[C]`) Single Authentication Flow
- **Choice**: When Bulldozer is triggered on an avenue containing one or more elevated targets, it prompts for the sudo password once up front.
- **Rationale**: Prevents prompting the user repeatedly for each elevated category. If the user cancels the sudo prompt, Bulldozer gracefully executes all user-level purges (Docker, package caches, temporary files) and leaves elevated targets on the avenue.

## System Architecture & Data Flow

```
+-----------------------------------------------------------------------------------+
|                         SUDO DEMOLITION DATA FLOW                                 |
+-----------------------------------------------------------------------------------+

   [User Action: Demolish Target (d/x/Delete)]
                     |
                     v
             requires_sudo?
             /            \
       (No) /              \ (Yes)
           v                v
   [Normal Confirm]    [In-App Sudo Password Modal]
           |                        |
           |             (User enters password & Enter)
           |                        |
           v                        v
   [PurgeAction::execute]  [PurgeAction::execute_with_sudo(password)]
                                    |
                                    +---> Command::new("sudo")
                                    |        .args(["-S", cmd, ...])
                                    |        .stdin(Stdio::piped())
                                    v
                            [Stream password to stdin]
                                    |
                                    v
                            [Zeroize password buffer]
                                    |
                                    v
                          Check exit status & stderr
                                    |
                    +---------------+---------------+
                    |                               |
             (Success 0)                    (Failure / Bad Pass)
                    v                               v
         [Remove building node]             [Show Error Banner]
         [Update total capacity]            [Retain target node]
```

## Risks / Trade-offs

- **[Risk] Password persistence in RAM** → **Mitigation**: Password buffers are contained only within the active modal state and explicitly overwritten/cleared immediately after being written to the child process stdin.
- **[Risk] Process deadlock on incorrect sudo password** → **Mitigation**: `sudo -S` exits immediately with status 1 on authentication failure when stdin is closed. The implementation closes the child stdin handle immediately after writing the password string to prevent hanging.
- **[Risk] Slow directory probing during Scrapyard survey** → **Mitigation**: New probes use shallow scans or command-based size checks (e.g. `journalctl --disk-usage` and checking specific version directories under `~/.local/share/cockpit-tools/updates`) rather than unbounded recursive traversal.
