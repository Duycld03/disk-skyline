# Tasks

## 1. Internationalization Core Subsystem

- [x] 1.1 Implement `Language` enum (`Vi`, `En`) in `src/i18n.rs` with `clap::ValueEnum`, `Display`, and toggle helper.
- [x] 1.2 Implement OS locale auto-detection from environment variables (`LC_ALL`, `LANG`) and complete dictionary accessors for all UI strings (avenue, HUD, building categories, bunkers, system protection, demolition).
- [x] 1.3 Write unit tests in `src/i18n.rs` verifying dictionary keys, fallback logic, and locale toggling.

## 2. TUI & CLI Bilingual Integration

- [x] 2.1 Update `App` in `src/app.rs` to hold `language: Language` state and intercept `L` key to toggle language at runtime.
- [x] 2.2 Refactor `src/ui.rs` and `src/render.rs` to consume `app.language` for header, avenue titles, building details, HUD pane, and navigation controls.
- [x] 2.3 Localize demolition modal dialogs, status messages, and rejection notifications in `src/demolish.rs` and `src/app.rs`.
- [x] 2.4 Add `--lang` / `-l` CLI argument in `src/main.rs`, update startup screen, and pass detected/specified language to `App::new`.
- [x] 2.5 Update existing unit and integration test assertions in `tests/` and `src/` to remain deterministic across language settings.

## 3. Repository Hygiene & Documentation

- [x] 3.1 Create `.gitignore` to strictly exclude `/target/`, `**/*.rs.bk`, `.omp/`, `*.log`, `.DS_Store`, and editor caches.
- [x] 3.2 Update `Cargo.toml` with `license = "MIT"`, `description`, `repository`, `readme`, `keywords`, and `categories`.
- [x] 3.3 Create MIT `LICENSE` file.
- [x] 3.4 Create comprehensive `README.md` featuring ASCII art banners, live demonstration snapshots, keybindings table, CLI options, and architecture notes.

## 4. GitHub Actions CI/CD Workflows

- [x] 4.1 Create `.github/workflows/ci.yml` matrix workflow testing `ubuntu-latest`, `windows-latest`, and `macos-latest` with `cargo check`, `cargo test`, and `cargo clippy`.
- [x] 4.2 Create `.github/workflows/release.yml` for tag `v*` releases, compiling targets (`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`), computing SHA-256 checksums, and uploading assets to GitHub Release.
- [x] 4.3 Run `cargo test` locally to verify 100% test pass rate across all modules.

## 5. Git Repository Setup & GitHub Publishing

- [x] 5.1 Initialize git repository (`git init -b main`) and verify `git status` honors `.gitignore`.
- [x] 5.2 Create initial commit and publish public repository using GitHub CLI (`gh repo create Duycld03/disk-skyline --public --source=. --remote=origin --push`).
