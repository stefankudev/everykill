# everykill

CLI tool to recursively scan directories and find/delete dependency folders across all languages and ecosystems. Similar to npkill but supports **all** ecosystems, not just Node.js.

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run in development (launches interactive TUI)
cargo run

# Release build
cargo build --release

# Run tests
cargo test

# Run with specific args (plain-text mode)
everykill --no-tui -t nodejs,rust -s size
```

## Project Overview

- **Language**: Rust
- **Type**: Interactive CLI tool with full-screen TUI
- **Purpose**: Find and optionally delete dependency/build folders (node_modules, target/, vendor/, etc.) to free disk space
- **Target Users**: Developers with many projects across multiple ecosystems

## Architecture

### Directory Structure

```
everykill/
├── src/                  # Rust source
│   ├── main.rs           # Binary entry point
│   ├── lib.rs            # Library entry, calls run() → TUI or plain-text
│   ├── args.rs           # CLI argument parsing with clap
│   ├── config/           # Configuration & ecosystem loading
│   │   ├── mod.rs
│   │   └── ecosystem.rs  # Ecosystem struct, loading, pattern matching
│   ├── scanner/          # Directory scanning & size calculation
│   │   ├── mod.rs
│   │   ├── dir.rs        # Directory traversal
│   │   └── size.rs       # Parallel size calculation
│   ├── size_util.rs      # Size formatting utility
│   ├── deleter.rs        # Folder deletion logic
│   └── ui/               # TUI components
│       ├── mod.rs
│       ├── ascii.rs      # ASCII art selection & rendering
│       ├── app.rs        # AppState, ScanEvent, state machine
│       ├── tui.rs        # Terminal init/restore, event loop, scan thread
│       └── widgets/
│           ├── mod.rs
│           ├── header.rs # ASCII art banner widget
│           ├── list.rs   # Scrollable folder table widget
│           ├── footer.rs # Status bar widget
│           └── filter.rs # Ecosystem filter popup widget
├── ecosystems/           # Language/ecosystem dependency definitions
│   ├── nodejs.json       # One file per ecosystem
│   ├── rust.json
│   └── ...               # ~60 ecosystem files total
├── assets/ascii/         # ASCII art for different terminal widths
│   ├── 44.txt            # 44 columns (smallest)
│   └── 68.txt            # 68 columns (largest)
├── docs/
│   ├── PROJECT_PLAN.md   # Implementation status & architecture
│   └── ECOSYSTEMS.md     # Full ecosystem reference
├── clippy.toml           # Clippy lint configuration
├── Cargo.toml            # Rust package manifest
└── Cargo.lock            # Dependency lock file
```

### Key Components

#### `src/main.rs` & `src/lib.rs`
- Entry point pattern: `main.rs` calls `everykill::run()`
- `run()` branches: TUI mode by default, `run_plain()` when `--no-tui` is set
- All business logic in the library crate for testability

#### `src/args.rs`
- `Args` struct derived from `clap::Parser`
- Supports all CLI flags: `-d`, `-t`, `--all`, `-g`, `-E`, `-x`, `--depth`, `--no-recursive`, `-f`, `-s`, `-e`, `-D`, `--no-tui`
- Helper methods: `get_scan_path()`, `get_depth_limit()`, `get_ecosystems()`, `get_excluded_dirs()`, `should_include_globals()`

#### `src/config/ecosystem.rs`
- `Ecosystem` struct with `name`, `local`, `global` fields
- `DiscoveredFolder` struct with `path`, `ecosystem`, `size_bytes`, `selected`
- `load_ecosystems()` — loads all ecosystems from `ecosystems/*.json`
- `Ecosystem::matches_folder_with_globals()` — pattern matching for folder names

#### `src/scanner/dir.rs`
- `scan_directory()` — walks directory tree, finds matching folders
- Uses `walkdir` for traversal
- Supports exclude directories, exclude hidden, max depth parameters
- Skips hidden dirs (`.git/`, `.svn/`, `.hg/`) and `.cache` directories
- Tracks inodes to avoid duplicates (Unix)

#### `src/scanner/size.rs`
- `calculate_size(path)` — recursive folder size calculation
- `calculate_sizes(folders)` — parallel size calculation using `rayon`

#### `src/size_util.rs`
- `format_size(bytes)` — converts bytes to human-readable format (B, KB, MB, GB, TB, PB, EB)
- `Size` struct with `value` and `unit`
- Dynamic precision: 0 decimals (≥100), 1 decimal (≥10), 2 decimals (<10)

#### `src/deleter.rs`
- `delete_folders()` — deletes selected folders; supports dry-run
- `DeleteSummary` struct with `deleted_count`, `freed_bytes`, `errors`
- `print_delete_summary()` — prints deletion summary (used in plain-text mode)

#### `src/ui/ascii.rs`
- `get_ascii_art(terminal_width)` — selects appropriate art: `None` if `< 44`, narrow if `44–68`, wide if `> 68`
- `print_centered_art(terminal_width)` — renders centered ASCII banner to stdout
- `get_terminal_width()` — gets current terminal width via crossterm
- Art files compiled into binary via `include_str!()`

#### `src/ui/app.rs`
- `AppState` — full application state: folders, cursor, scroll, selections, mode, filters, dry-run flag, scan state, totals
- `ScanState` enum: `Scanning` / `Complete` / `Error(String)`
- `AppMode` enum: `Normal` / `FilterPopup` / `ConfirmDelete`
- `ScanEvent` enum: `FolderFound` / `SizeUpdated` / `Done` / `Error` — sent from background scan thread
- State mutation methods: `toggle_selection`, `select_all`, `deselect_all`, `cursor_up/down`, `page_up/down`, `jump_to_top/bottom`, `toggle_ecosystem_filter`, etc.

#### `src/ui/tui.rs`
- `run_tui(args)` — public entry point; installs panic hook, sets up terminal, runs event loop, restores terminal
- Terminal management: `enable_raw_mode`, `EnterAlternateScreen`, `EnableMouseCapture` on enter; full restore on exit or panic
- Background scan: spawns `std::thread` that streams `ScanEvent` via `mpsc::channel`
- Event loop: ~60 fps (`16ms` tick); drains `ScanEvent` channel, renders, polls `crossterm` for key/mouse events
- Key handling: Normal / FilterPopup / ConfirmDelete modes fully mapped (including vim keys `j`/`k`)
- Mouse handling: click to focus, click checkbox to toggle, scroll wheel navigation
- Deletion execution: calls `deleter::delete_folders`, removes deleted paths from state, shows status message

#### `src/ui/widgets/`
- `header.rs` — renders ASCII art banner centred in cyan; auto-hides when terminal `< 44` wide
- `list.rs` — 4-column scrollable `Table` (checkbox / path / ecosystem / size); cursor row highlighted; size shows `…` while scanning
- `footer.rs` — 3-line status bar adapting to Normal / ConfirmDelete / FilterPopup modes; shows scan progress, selection stats, keybind hints
- `filter.rs` — floating ecosystem-filter popup (60% × 70% of terminal), rendered with `Clear` over the list

#### `ecosystems/*.json`
- One file per language/ecosystem
- Schema:
```json
{
  "name": "Display Name",
  "local": ["path/", "other/"],
  "global": ["~/.cache/path"]
}
```
- Loaded via glob on startup
- `local` = per-project dependency folders (scanned from current directory)
- `global` = user-level caches (included only with `-g` / `--global`)

### CLI Flags

| Flag                     | Description                                       | Default   |
| ------------------------ | ------------------------------------------------- | --------- |
| `-d, --directory <PATH>` | Directory to scan                                 | `.`       |
| `-t, --target <LANGS>`   | Ecosystems to scan (comma-separated)              | All local |
| `--all`                  | Include all ecosystems                            | `false`   |
| `-g, --global`           | Include global/user-level caches                  | `false`   |
| `-E, --exclude <DIRS>`   | Exclude directories by name (comma-separated)     | None      |
| `-x, --exclude-hidden`   | Exclude hidden directories                        | `false`   |
| `--no-recursive`         | Don't scan subdirectories (alias for `--depth 0`) | `false`   |
| `--depth <N>`            | Maximum directory depth (0 = current only)        | Unlimited |
| `-f, --full`             | Scan from home directory                          | `false`   |
| `-s, --sort <BY>`        | Sort by `size` or `path` (plain-text mode)        | None      |
| `-e, --show-errors`      | Show error messages                               | `false`   |
| `-D, --delete`           | Delete all found folders (plain-text mode)        | `false`   |
| `--no-tui`               | Disable interactive TUI; print plain text         | `false`   |
| `-h, --help`             | Show help                                         | -         |
| `-v, --version`          | Show version                                      | -         |

### TUI Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate list |
| `Page Up` / `Page Down` | Scroll by page |
| `Home` / `End` | Jump to top / bottom |
| `Space` | Toggle selection on current row |
| `a` | Select all visible folders |
| `n` | Deselect all |
| `d` | Toggle dry-run mode |
| `f` | Open ecosystem filter popup |
| `Enter` | Confirm deletion of selected folders |
| `q` / `Q` / `Esc` | Quit |

Mouse: click row to focus, click `[ ]` to toggle selection, scroll wheel to navigate.

## Design Decisions

| Decision                              | Rationale                                        | Status |
| ------------------------------------- | ------------------------------------------------ | ------ |
| Per-ecosystem JSON files              | Easy contribution without code changes           | Done   |
| Rust                                  | Single binary, no runtime needed, fast           | Done   |
| ratatui for TUI                       | Full keyboard/mouse support, npkill-style        | Done   |
| Parallel scanning with rayon          | Fast directory traversal                         | Done   |
| Background scan via std::thread+mpsc  | Live results without async runtime               | Done   |
| Skip hidden dirs (.git, etc.)         | Avoid scanning version control                   | Done   |
| Size calculation after scan           | Separate phases for clarity                      | Done   |
| `--no-tui` plain-text fallback        | Scriptable without removing TUI as default       | Done   |
| Pre-compiled ASCII art                | No runtime generation needed                     | Done   |
| `ecosystems/` not `languages/`        | Avoids i18n/l10n confusion                       | Done   |
| Panic hook restores terminal          | Terminal never left in raw mode on crash         | Done   |

## CI/CD & Distribution

### Publishing Targets

| Platform         | Method                              | Status  |
| ---------------- | ----------------------------------- | ------- |
| crates.io        | `cargo publish`                     | Planned |
| GitHub Releases  | Pre-built binaries (x86, ARM, musl) | Planned |
| Homebrew         | Tap repo + formula                  | Planned |
| AUR (Arch Linux) | User-contributed                    | Planned |
| Windows          | winget, scoop                       | Future  |

### Pipeline

Uses `cargo-dist` for automated multi-platform builds:

1. On tag push: `cargo dist build --all`
2. Create GitHub Release with artifacts
3. Publish to crates.io
4. Update Homebrew tap (auto-PR)

### Release Process

1. Update version in `Cargo.toml`
2. Create git tag: `git tag v0.1.0`
3. Push tag: `git push origin v0.1.0`
4. GitHub Actions builds and publishes to all targets

## Contributing

### Commit Style (Conventional Commits)

Format: `<type>(<scope>): <description>`

Types:
- `feat` - New feature
- `fix` - Bug fix
- `chore` - Maintenance, deps, tooling
- `refactor` - Code restructuring
- `style` - Formatting, styling
- `docs` - Documentation
- `test` - Tests
- `perf` - Performance

Examples:
```bash
git commit -m "feat(tui): add ecosystem filter popup"
git commit -m "fix(scanner): handle permission errors"
git commit -m "chore(deps): update ratatui to 0.30"
git commit -m "feat(ecosystems): add swift support"
```

### Adding a New Ecosystem

1. Create `ecosystems/<name>.json` with schema:
```json
{
  "name": "Display Name",
  "local": ["path/", "other/"],
  "global": ["~/.cache/path"]
}
```
2. Add file to git
3. Commit with type `feat(ecosystems): add <name> support`

### Adding ASCII Art for New Width

1. Create `assets/ascii/XXX.txt` with ASCII banner for that width
2. Update `src/ui/ascii.rs` match arms to include new width
3. Commit with type `feat(ascii): add width XXX art`

## Roadmap

- [x] Create actual ASCII art for each width
- [x] Implement directory scanner module
- [x] Implement ecosystem JSON loader
- [x] Add folder size calculation
- [x] Add CLI args via clap
- [x] Add size formatting utility
- [x] Add clippy linting configuration
- [x] Add deletion logic (CLI mode)
- [x] Implement TUI with ratatui
- [x] Add keyboard navigation (arrows, space, enter, vim keys)
- [x] Add mouse support (click, scroll)
- [x] Background scan thread with live progress
- [x] Ecosystem filter popup
- [x] Deletion confirmation prompt with dry-run mode
- [x] Panic-safe terminal restore
- [ ] Cross-platform testing (Windows)
- [ ] Set up cargo-dist pipeline
- [ ] First release

## Reference

See [docs/ECOSYSTEMS.md](./docs/ECOSYSTEMS.md) for the complete list of supported ecosystems and their dependency paths.
