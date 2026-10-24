# Design

## Context

`disk-skyline` supports running against any target directory, including system root (`/` on Linux or `C:\` on Windows). Scanning and file operations at system root present unique stability and data safety challenges. See `proposal.md` for motivation and `specs/system-protection/spec.md` for requirements.

## Goals / Non-Goals

**Goals:**
- Skip virtual kernel pseudo-filesystems during disk scanning to avoid infinite streams, loops, and skewed statistics.
- Automatically identify OS core infrastructure and tag nodes with `is_system_protected = true`.
- Disable demolition (`d` / `Delete`) for system-protected buildings, providing instant refusal feedback.
- Visually mark protected buildings on the skyline and update HUD action guides.

**Non-Goals:**
- Custom file-permission manipulation or elevation (e.g. running `sudo` or modifying ACLs).
- Full read-only mode for user-owned subdirectories within the system.

## Decisions

### 1. Model Extension: `is_system_protected` Flag
- **Decision**: Add `pub is_system_protected: bool` to `BuildingNode`.
- **Rationale**: Carrying protection status on the node allows O(1) checks in both the rendering layer and the event handler without repeated path string comparisons.

### 2. Path Classification & Exclusion in `scanner.rs`
- **Decision**: Introduce `is_virtual_pseudo_fs(path: &Path) -> bool`:
  - On Unix: Skip directory paths matching `/proc`, `/sys`, `/dev`, `/run` (exact match or direct root prefix).
  - On Windows: Skip `System Volume Information`, `$Recycle.Bin`, and junction reparse points.
- **Rationale**: Completely eliminating these paths from recursive `read_dir` calls prevents 128 TB virtual allocation artifacts (from `/proc/kcore`) and avoids device file stream hangs.

### 3. System Landmark Protection Rules in `model.rs`
- **Decision**: Introduce `is_os_critical_path(path: &Path) -> bool`:
  - **Linux / Unix**:
    - Root `/` itself.
    - `/bin`, `/sbin`, `/lib`, `/lib64`, `/usr`, `/etc`, `/boot`, `/root`.
  - **Windows**:
    - `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)`, `C:\ProgramData`.
    - `pagefile.sys`, `hiberfil.sys`, `swapfile.sys`, `dumpstack.log`.
    - `Recovery`, `System Volume Information`.
- **Rationale**: Explicitly blacklisting known core OS directories prevents catastrophic OS breakage even if the user runs `disk-skyline` with root or administrator privileges.

### 4. Demolition Interception in `App::handle_key`
- **Decision**: In `App::handle_key`, before instantiating `DemolishState`:
  ```rust
  if let Some(child) = self.current_selected_child() {
      if child.is_system_protected {
          self.status_message = Some((
              "[!] TỪ CHỐI THAO TÁC: Công trình hạ tầng thiết yếu của Hệ điều hành. Phá dỡ bị cấm tuyệt đối!".into(),
              true,
          ));
          return;
      }
      self.demolish_state = Some(DemolishState::new(child));
  }
  ```
- **Rationale**: Immediate interception prevents the demolition modal from opening altogether, guaranteeing that no key combination can bypass the lock.

### 5. UI and HUD Feedback
- **Decision**:
  - Building Facade: Protected buildings display a `[LOCKED]` roof pattern and red warning emblem.
  - HUD Pane: Displays `[KHÓA BẢO VỆ HỆ THỐNG]` in bold red.
  - Key Legend: Replaces `[d/Del] Phá dỡ` with `[d] Khóa hệ thống (Protected)` in dark gray when focusing on a protected building.

## Risks / Trade-offs

- **[Risk: User wishes to clean unneeded logs under `/var/log` or build files in `/tmp`]**
  $\rightarrow$ *Mitigation*: Do not mark `/tmp` or `/var` root as system protected, only lock critical core directories like `/etc`, `/usr`, `/boot`.
- **[Risk: Different platform path separators (`/` vs `\`)]**
  $\rightarrow$ *Mitigation*: Use `Path::components()` and canonicalized paths rather than raw string prefix matching to handle cross-platform path normalization cleanly.
