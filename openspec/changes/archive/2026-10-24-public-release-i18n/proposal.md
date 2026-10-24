# Proposal

## Why

The `disk-skyline` project is ready for open-source distribution on GitHub, but currently lacks internationalization (all UI strings are hardcoded in Vietnamese), repository hygiene files (.gitignore, README, LICENSE), automated cross-platform CI/CD testing, and automated GitHub Release artifact publishing. Preparing the project with bilingual support (Vietnamese and English), strict repository sanitization, and release workflows enables global adoption and seamless distribution.

## What Changes

- **Repository Hygiene & Sanitization**: Add comprehensive `.gitignore` (ignoring `target/`, temporary files, and internal agent harness files), an MIT `LICENSE`, a feature-rich `README.md` with ASCII art and usage guides, and complete `Cargo.toml` package metadata (license, description, repository, keywords).
- **Internationalization (i18n)**: Introduce a centralized, type-safe i18n dictionary supporting Vietnamese (`vi`) and English (`en`). Auto-detect system locale (`LANG`/`LC_ALL`), provide a CLI `--lang` / `-l` override, and enable runtime toggle between languages via the `L` key in the TUI without restarting.
- **TUI & HUD Localization**: Adapt all UI headers, street navigation breadcrumbs, building inspections, restricted bunker tags, system-protection locks, and demolition modals to display localized text dynamically.
- **GitHub Actions CI/CD**:
  - Continuous Integration (`.github/workflows/ci.yml`): Run `cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check` across Linux, macOS, and Windows on every push and pull request.
  - Release Pipeline (`.github/workflows/release.yml`): Trigger on tag push (`v*`), cross-compile release binaries for Linux (`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`), Windows (`x86_64-pc-windows-msvc`), and macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`), package into `.tar.gz` and `.zip` with SHA-256 checksums, and publish release assets to GitHub Releases.
- **Public GitHub Repository Setup**: Initialize git, establish remote tracking with GitHub CLI (`gh`), create the public repository under the user account, and push the initial clean commit.

## Capabilities

### New Capabilities
- `i18n`: Centralized bilingual string dictionary (Vietnamese and English) with OS environment locale detection, CLI override, and runtime locale state management.

### Modified Capabilities
- `disk-skyline`: Extended to support dynamic UI re-rendering upon language toggle (`L` key), CLI `--lang` parameter integration, and fully localized HUD, avenue, and modal text.

## Impact

- **Affected Code**: `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/ui.rs`, `src/render.rs`, `src/demolish.rs`, new `src/i18n.rs`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.gitignore`, `README.md`, `LICENSE`.
- **Breaking Changes**: None. Default CLI behavior preserves existing capabilities while defaulting to system locale or explicit `--lang` flag.
