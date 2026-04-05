# everykill

CLI tool to recursively scan directories and find/delete dependency folders across all languages and ecosystems. Similar to npkill but supports **all** ecosystems, not just Node.js.

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run in development
cargo run

# Release build
cargo build --release

# Run tests
cargo test

# Run with specific args
everykill -t nodejs,rust -s size
```

## Project Overview

- **Language**: Rust
- **Type**: Interactive CLI tool with TUI
- **Purpose**: Find and optionally delete dependency/build folders (node_modules, target/, vendor/, etc.) to free disk space
- **Target Users**: Developers with many projects across multiple ecosystems

## Architecture

### Directory Structure

```
everykill/
├── src/                  # Rust source
│   ├── main.rs           # Binary entry point
│   ├── lib.rs            # Library entry, calls run()
│   ├── args.rs           # CLI argument parsing with clap
│   ├── config/           # Configuration & ecosystem loading
│   │   ├── mod.rs
│   │   └── ecosystem.rs  # Ecosystem struct, loading, pattern matching
│   ├── scanner/          # Directory scanning & size calculation
│   │   ├── mod.rs
│   │   ├── dir.rs        # Directory traversal
│   │   └── size.rs       # Parallel size calculation
│   ├── size_util.rs      # Size formatting utility
│   └── ui/               # TUI components (planned)
│       ├── mod.rs
│       └── ascii.rs      # ASCII art selection & rendering
├── ecosystems/           # Language/ecosystem dependency definitions
│   ├── nodejs.json       # One file per ecosystem
│   ├── rust.json
│   └── ...               # ~60 ecosystem files total
├── assets/ascii/         # ASCII art for different terminal widths
│   ├── 44.txt            # 44 columns (smallest)
│   └── 68.txt            # 68 columns (largest)
├── clippy.toml           # Clippy lint configuration
├── Cargo.toml            # Rust package manifest
└── Cargo.lock            # Dependency lock file
```

### Key Components

#### `src/main.rs` & `src/lib.rs`
- Entry point pattern: `main.rs` calls `everykill::run()`
- All business logic in library for testability
- Uses `clap` for CLI argument parsing

#### `src/args.rs`
- `Args` struct derived from `clap::Parser`
- Supports all CLI flags: `-d`, `-t`, `--all`, `-g`, `-E`, `-x`, `--depth`, `--no-recursive`, `-f`, `-s`, `-e`
- Helper methods for filtering ecosystems, getting scan paths, etc.

#### `src/config/ecosystem.rs`
- `Ecosystem` struct with `name`, `local`, `global` fields
- `DiscoveredFolder` struct with `path`, `ecosystem`, `size_bytes`, `selected`
- `load_ecosystems()` - loads all ecosystems from `ecosystems/*.json`
- `load_ecosystem(name)` - lazy load single ecosystem
- `Ecosystem::matches_folder()` - pattern matching for folder names
- `Ecosystem::matches_folder_with_globals()` - supports include_globals flag

#### `src/scanner/dir.rs`
- `scan_directory()` - walks directory tree, finds matching folders
- Uses `walkdir` for traversal
- Supports exclude directories, exclude hidden, max depth parameters
- Skips hidden dirs (`.git/`, `.svn/`, `.hg/`) and `.cache` directories
- Tracks inodes to avoid duplicates

#### `src/scanner/size.rs`
- `calculate_size(path)` - recursive folder size calculation
- `calculate_sizes(folders)` - parallel size calculation using `rayon`

#### `src/size_util.rs`
- `format_size(bytes)` - converts bytes to human-readable format (B, KB, MB, GB, TB, PB, EB)
- `Size` struct with `value` and `unit`
- Dynamic precision: 0 decimals (≥100), 1 decimal (≥10), 2 decimals (<10)

#### `src/deleter.rs`
- `delete_folders()` - deletes selected folders
- `DeleteSummary` struct with `deleted_count`, `freed_bytes`, `errors`
- `print_delete_summary()` - prints deletion summary

#### `src/ui/ascii.rs`
- `get_ascii_art(terminal_width)` - selects appropriate art based on terminal size
- `print_centered_art(terminal_width)` - renders centered ASCII banner
- `get_terminal_width()` - gets current terminal width via crossterm
- Art files compiled into binary via `include_str!()`

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
- `global` = user-level caches (typically not deleted by default)

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
| `-s, --sort <BY>`        | Sort by `size` or `path`                          | None      |
| `-e, --show-errors`      | Show error messages                               | `false`   |
| `-D, --delete`           | Delete found folders                              | `false`   |
| `-h, --help`             | Show help                                         | -         |
| `-v, --version`          | Show version                                      | -         |

### Planned UI Flow

**Note:** CLI deletion is now implemented. The TUI will provide interactive selection before deletion.

1. Display ASCII art banner (centered based on terminal width)
2. Scan directory tree for matching dependency folders
3. Display list of found folders with ecosystem type and sizes
4. Arrow keys to navigate, Space to select, Enter to delete
5. Confirmation prompt before deletion
6. Show summary of freed space

## Design Decisions

| Decision                       | Rationale                                 | Status  |
| ------------------------------ | ----------------------------------------- | ------- |
| Per-ecosystem JSON files       | Easy contribution without code changes    | Done    |
| Rust                           | Single binary, no runtime needed, fast    | Done    |
| Ratatui for TUI                | Full keyboard/mouse support, npkill-style | Planned |
| Parallel scanning with rayon   | Fast directory traversal                  | Done    |
| Skip hidden dirs (.git, etc.)  | Avoid scanning version control            | Done    |
| Size calculation after scan    | Separate phases for clarity               | Done    |
| CLI deletion before TUI        | Standalone usable without TUI             | Done    |
| Pre-compiled ASCII art         | No runtime generation needed              | Done    |
| `ecosystems/` not `languages/` | Avoids i18n/l10n confusion                | Done    |

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
git commit -m "feat(scanner): add directory traversal"
git commit -m "fix(delete): handle permission errors"
git commit -m "chore: add clap for CLI args"
git commit -m "style(ui): center ASCII art banner"
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

1. Create or update `assets/ascii/XXX.txt` with ASCII banner
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
- [ ] Implement TUI with ratatui
- [ ] Add keyboard navigation (arrows, space, enter)
- [ ] Cross-platform testing (Windows)
- [ ] Set up cargo-dist pipeline
- [ ] First release

## Reference

See [ECOSYSTEMS.md](./docs/ECOSYSTEMS.md) for complete list of supported ecosystems and their dependency paths.
