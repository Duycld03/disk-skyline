# Tasks

## 1. Model Updates & System Classification

- [x] 1.1 Add `is_system_protected` boolean field to `BuildingNode` in `src/model.rs` and update constructors, verifying existing model unit tests pass
- [x] 1.2 Implement `is_os_critical_path` helper function in `src/model.rs` identifying core system directories and files on Linux and Windows, verifying unit tests cover Linux (`/usr`, `/etc`, `/boot`) and Windows targets

## 2. Kernel Pseudo-Filesystem Exclusion in Scanner

- [x] 2.1 Implement `is_virtual_pseudo_fs` filter in `src/scanner.rs` to detect and skip `/proc`, `/sys`, `/dev`, `/run`, and Windows system volume metadata paths, verifying unit tests
- [x] 2.2 Wire exclusion filter into recursive directory scanner in `src/scanner.rs` so virtual paths are completely bypassed during traversal, verifying unit tests on fixture trees

## 3. Demolition Lockdown & UI Feedback

- [x] 3.1 Intercept demolition key handling in `src/app.rs` when focused on a system-protected building, displaying a refusal notification without activating `demolish_state`, and verify unit tests
- [x] 3.2 Update HUD and facade rendering in `src/ui.rs` and `src/render.rs` to display `[KHÓA BẢO VỆ HỆ THỐNG]` status and grayed-out demolition prompt, verifying buffer render tests

## 4. Integration Verification

- [x] 4.1 Add an end-to-end integration test in `tests/system_protection_test.rs` verifying that system-protected nodes reject demolition and virtual paths are skipped
- [x] 4.2 Run full test suite (`cargo test`) to ensure zero regressions across all modules
