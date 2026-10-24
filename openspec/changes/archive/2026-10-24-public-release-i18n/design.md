# Design

## Context

See `proposal.md` for background. The existing application hardcodes Vietnamese strings directly inside UI rendering logic (`src/ui.rs`), modal prompts (`src/demolish.rs`), and scanner status prints (`src/main.rs`). The workspace also lacks Git tracking (`.git`), standard packaging metadata, and GitHub Actions CI/CD automation.

## Goals / Non-Goals

**Goals:**
- Provide instantaneous bilingual support (Vietnamese & English) with zero runtime file dependencies.
- Enable automatic locale detection at launch, CLI flag override (`--lang`), and dynamic runtime switching via key `L`.
- Secure the codebase with `.gitignore` to prevent committing build caches (`target/`) or agent harness scripts (`.omp/`).
- Package and document the project with MIT `LICENSE`, `README.md`, and complete `Cargo.toml` metadata.
- Automate multi-platform CI verification and tag-triggered cross-platform binary releases on GitHub.
- Publish the repository to GitHub as a public repo under `Duycld03`.

**Non-Goals:**
- Heavy external localization runtime dependencies (e.g. `fluent` or external `.po`/`.json` file loaders).
- Support for RTL (Right-to-Left) languages or non-Latin script rendering in this iteration.

## Decisions

### 1. Compile-Time Type-Safe i18n Dictionary (`src/i18n.rs`)

**Decision:** Create a lightweight, native Rust module `src/i18n.rs` with `enum Language { Vi, En }` and typed accessor methods for all UI strings.
**Rationale:**
- Zero runtime overhead or filesystem I/O needed to load translations.
- Compile-time exhaustiveness: missing a translation in one language causes a compiler error.
- Implements `clap::ValueEnum` for clean CLI argument parsing (`--lang vi` / `--lang en`).
- Automatic environment fallback: inspects `std::env::var("LC_ALL").or_else(|_| std::env::var("LANG"))`; if it starts with `vi`, selects `Language::Vi`, otherwise defaults to `Language::En`.

### 2. Runtime Language State in App Loop

**Decision:** Store `pub language: Language` on `App` state, toggleable on keypress `L`.
**Rationale:**
- Users can switch languages mid-session without restarting or losing the current street navigation cursor and in-memory directory tree.
- `ui::draw_ui` passes `app.language` to child rendering functions (`render_header`, `render_hud`, `render_demolish_modal`).

### 3. Repository Hygiene & Public Release Artifacts

**Decision:**
- Create `.gitignore` ignoring: `/target/`, `Cargo.lock` (retained or ignored; for a CLI tool, keeping `Cargo.lock` ensures reproducible CI builds), `.omp/`, `*.log`, `.DS_Store`, `Thumbs.db`.
- Create `LICENSE` with standard MIT terms.
- Create `README.md` featuring ASCII city skyline banner, live snapshot demonstration, feature highlights, keybindings table, CLI flags reference, and installation guides.
- Update `Cargo.toml` with `description = "Interactive 2D Disk Skyline & TUI Visualizer in Rust"`, `license = "MIT"`, `repository = "https://github.com/Duycld03/disk-skyline"`, `readme = "README.md"`, `keywords`, and `categories`.

### 4. Cross-Platform GitHub Actions CI/CD

**Decision:**
- `.github/workflows/ci.yml`:
  - Triggers on `push` and `pull_request` to `main`.
  - Matrix runner: `ubuntu-latest`, `windows-latest`, `macos-latest`.
  - Steps: `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `cargo check`, `cargo test --all`, `cargo clippy -- -D warnings`.
- `.github/workflows/release.yml`:
  - Triggers on tag push `v*` (e.g. `v0.1.0`).
  - Matrix targets:
    - `x86_64-unknown-linux-gnu` (Ubuntu runner)
    - `x86_64-unknown-linux-musl` (Ubuntu runner with `musl-tools`)
    - `x86_64-pc-windows-msvc` (Windows runner)
    - `x86_64-apple-darwin` (macOS runner)
    - `aarch64-apple-darwin` (macOS runner)
  - Packages each release with `LICENSE` and `README.md` into `.tar.gz` (Unix) or `.zip` (Windows).
  - Generates `SHA256SUMS.txt`.
  - Publishes to GitHub Releases via `softprops/action-gh-release@v2`.

### 5. Git Repository Initialization

**Decision:**
- Initialize `git init -b main`.
- Stage all tracked files, create initial commit `feat: initial release of disk-skyline with bilingual support`.
- Execute `gh repo create Duycld03/disk-skyline --public --source=. --remote=origin --push`.

## Risks / Trade-offs

- **Risk:** Windows / macOS target cross-compilation nuances in GitHub Actions.
  - *Mitigation:* Use native matrix runners (`windows-latest`, `macos-latest`, `ubuntu-latest`) to compile native binaries rather than emulated cross-compilation where possible.
- **Risk:** Existing unit tests assert Vietnamese strings in HUD or rejection messages.
  - *Mitigation:* Ensure unit tests verify localized outputs against the active language parameter or default to the test's expected language.
