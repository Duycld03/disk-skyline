# Proposal

## Why

Developers frequently run one-off Node.js CLIs via `npx` (e.g. `npx create-next-app`, `npx tsx`, `npx prisma`). Over time, `~/.npm/_npx` silently accumulates extracted package artifacts and binaries, consuming gigabytes of disk space (e.g. ~755 MB observed on developer machines) without appearing in global `npm cache verify` or being caught by the current `~/.npm/_cacache` cleaner probe in Scrapyard.

## What Changes

- Add a dedicated probe in `src/cleaner/mod.rs` to detect and quantify reclaimable storage in `~/.npm/_npx`.
- Expose the detected junk as an individual cleanable building target (`id: "npx"`, Name: `NPX Ephemeral Cache`) in Scrapyard mode.
- Support single demolition (`[X]` / `Delete`) and batch Bulldozer demolition (`[C]`) targeting `~/.npm/_npx`.
- Re-create the empty `~/.npm/_npx` directory upon purge so future `npx` invocations do not fail on missing target directories.
- Add unit and regression tests verifying detection thresholding and safe deletion for the `_npx` cache.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `system-cleaner`: Broaden global package manager cache detection and purging requirements to include NPX ephemeral execution caches (`~/.npm/_npx`).

## Impact

- **User experience**: Scrapyard automatically surveys and surfaces `~/.npm/_npx` as an actionable, high-yield junk building when its size exceeds the threshold (>10 MB).
- **Disk recovery**: Allows developers to instantly reclaim hundreds of megabytes to gigabytes of ephemeral CLI caches.
- **Breaking changes**: None. The probe operates without sudo, purely within the user home directory.
