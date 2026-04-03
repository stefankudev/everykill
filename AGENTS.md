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
cargo run -- --all --dry-run
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
│   ├── lib.rs            # Library entry, calls ui::run()
│   └── ui/               # TUI components
│       ├── mod.rs        # Module exports
│       └── ascii.rs      # ASCII art selection & rendering
├── ecosystems/           # Language/ecosystem dependency definitions
│   ├── nodejs.json       # One file per ecosystem
│   ├── rust.json
│   └── ...               # ~60 ecosystem files total
├── ascii/                # ASCII art for different terminal widths
│   ├── width_025.txt     # 25 columns (smallest)
│   ├── width_050.txt
│   ├── width_075.txt
│   ├── width_100.txt
│   ├── width_125.txt
│   ├── width_150.txt
│   ├── width_175.txt
│   ├── width_200.txt
│   ├── width_225.txt
│   └── width_250.txt     # 250 columns (largest)
├── assets/fonts/         # Font files for ASCII art generation
│   ├── bold-killer/
│   ├── killer/
│   └── killer-tech/
├── Cargo.toml            # Rust package manifest
└── Cargo.lock            # Dependency lock file
```

### Key Components

#### `src/main.rs` & `src/lib.rs`
- Entry point pattern: `main.rs` calls `everykill::run()`
- All business logic in library for testability

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

### Planned CLI Flags

| Flag            | Description                                           |
| --------------- | ----------------------------------------------------- |
| `--all`         | Scan all ecosystems                                   |
| `--lang <name>` | Filter to specific ecosystem(s)                       |
| `--path <dir>`  | Specify root directory to scan (default: current dir) |
| `--dry-run`     | Preview what would be deleted (default: true)         |
| `--delete`      | Actually perform deletion                             |
| `--size`        | Show folder sizes in output                           |

### Planned UI Flow

1. Display ASCII art banner (centered based on terminal width)
2. Scan directory tree for matching dependency folders
3. Display list of found folders with ecosystem type and sizes
4. Arrow keys to navigate, Space to select, Enter to delete
5. Confirmation prompt before deletion
6. Show summary of freed space

## Design Decisions

| Decision                       | Rationale                                        |
| ------------------------------ | ------------------------------------------------ |
| Per-ecosystem JSON files       | Easy contribution without code changes           |
| Rust                           | Single binary, no runtime needed, fast           |
| Ratatui for TUI                | Full keyboard/mouse support, npkill-style        |
| Pre-compiled ASCII art         | No runtime generation needed                     |
| 10 ASCII art widths            | 25-250 cols in 25-col increments                 |
| `assets/` for fonts            | External assets live here (may be removed later) |
| `ecosystems/` not `languages/` | Avoids i18n/l10n confusion                       |

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

1. Create or update `ascii/width_XXX.txt` with 6-line ASCII banner
2. Update `src/ui/ascii.rs` match arms to include new width
3. Commit with type `feat(ascii): add width XXX art`

## Roadmap

High-level TODO (detail to be added as development progresses):

- [ ] Implement directory scanner module
- [ ] Implement ecosystem JSON loader
- [ ] Implement TUI with ratatui
- [ ] Add keyboard navigation (arrows, space, enter)
- [ ] Add deletion logic with confirmation
- [ ] Add CLI args via clap
- [ ] Add folder size calculation
- [ ] Create actual ASCII art for each width
- [ ] Implement --dry-run flag
- [ ] Implement --lang filter
- [ ] Cross-platform testing (Windows)
- [ ] Set up cargo-dist pipeline
- [ ] First release

## Reference

See [ECOSYSTEMS.md](./ECOSYSTEMS.md) for complete list of supported ecosystems and their dependency paths.
