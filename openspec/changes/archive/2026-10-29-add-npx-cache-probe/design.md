# Design

## Context

See `proposal.md - Why` for motivation. Currently, `src/cleaner/mod.rs` defines `probe_package_caches()` which surveys user-home package caches including `npm` (`~/.npm/_cacache`), `pip`, `uv`, `cargo`, `playwright`, and `nuget`. Node developers using `npx` accumulate binaries in `~/.npm/_npx`, but this path is currently skipped by all probes.

## Goals / Non-Goals

**Goals:**
- Provide a clean, dedicated probe for `~/.npm/_npx` in `probe_package_caches()` with minimal overhead.
- Present `npx` as an independent building in Scrapyard with clear visual naming and reclaimable size reporting.
- Ensure post-purge safety by keeping an empty `~/.npm/_npx` directory via the existing `PurgeAction::DirectoryDelete` contract.
- Maintain seamless integration with both single demolition (`[X]`) and batch Bulldozer demolition (`[C]`).

**Non-Goals:**
- Selective pruning of individual package hashes inside `_npx` (npx cache is disposable by nature; purging all ephemeral packages is the intended behavior).
- Managing global npm modules or `node_modules` in project workspaces.

## Decisions

### 1. Dedicated Target ID (`npx`) over Bundling into `npm`
- **Choice**: Introduce a separate `CleanableTarget` with `id: "npx"`, `name: "NPX Ephemeral Cache"`.
- **Rationale**: Keeps Scrapyard information transparent. Users often keep npm caches for faster builds, but are happy to wipe stale npx binaries. Bundling would obscure which path actually contains the bulk of disk space.
- **Alternative considered**: Expanding the `npm` target to delete both `_cacache` and `_npx` under one building. Rejected because users could not selectively inspect or clear npx without clearing npm dependencies cache.

### 2. Purge Action Mechanics
- **Choice**: Reuse `PurgeAction::DirectoryDelete(npx_path)`.
- **Rationale**: `PurgeAction::DirectoryDelete` already deletes the directory tree and then immediately invokes `std::fs::create_dir_all(path)`. This guarantees that subsequent `npx <command>` executions will not encounter missing directory errors.

### 3. Threshold Configuration
- **Choice**: 10 MB (`10 * 1024 * 1024` bytes).
- **Rationale**: Consistent with `npm` and `cargo` thresholds in `src/cleaner/mod.rs`. Ignores fresh, trivial npx runs while triggering on real disk clutter.

### 4. Privilege Boundary
- **Choice**: `requires_sudo: false`.
- **Rationale**: Stored in `$HOME/.npm/_npx`, strictly user-owned.

## Risks / Trade-offs

- **Risk**: User running an `npx` command concurrently during demolition.
  - **Mitigation**: Standard filesystem behavior; in-use open file descriptors on Linux may be deleted from directory entries without crashing, but subsequent executions will freshly re-download cleanly. No elevated privilege risk.
