# Tasks

## 1. NPX Cache Probe Implementation

- [x] 1.1 Implement probe for `~/.npm/_npx` in `src/cleaner/mod.rs` with `id: "npx"`, threshold 10 MB, and `requires_sudo: false`, verifying with unit tests that it produces a valid `CleanableTarget`.
- [x] 1.2 Wire the `npx` probe into `probe_package_caches()` in `src/cleaner/mod.rs` so it is collected by `survey_all_system_junk()`.

## 2. Unit & Integration Testing

- [x] 2.1 Add unit tests in `src/cleaner/mod.rs` covering threshold boundaries (sub-10MB skipped, >10MB detected, missing directory handled gracefully).
- [x] 2.2 Add test verifying `PurgeAction::DirectoryDelete` safely purges `_npx` contents and recreates the empty root folder.
- [x] 2.3 Run `cargo test --lib cleaner` and ensure all cleaner probe tests pass.

## 3. End-to-End Scrapyard Verification

- [x] 3.1 Verify detection on the live system by checking `survey_all_system_junk()` output or running `--scrapyard` mode.
