# everykill

The CLI tool to find and delete dependency folders across all languages and ecosystems. Similar to npkill but supports **all** ecosystems, not just Node.js.

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run (launches interactive TUI)
cargo run

# Release build
cargo build --release

# Run tests
cargo test
```

## Usage

```bash
# Interactive TUI (default) — browse, select, and delete
everykill

# Scan a specific directory
everykill -d ~/projects

# Target specific ecosystems
everykill -t nodejs,rust

# Include global/user-level caches
everykill -t nodejs --global

# Exclude directories by name
everykill -E "target,vendor"

# Sort by size (largest first) — applies in --no-tui mode
everykill --no-tui -s size

# Plain-text output (no TUI — useful for scripting/piping)
everykill --no-tui

# Delete without TUI confirmation
everykill --no-tui --delete

# Full help
everykill --help
```

## TUI Key Bindings

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

Mouse is also supported: click a row to focus it, click the `[ ]` checkbox to toggle selection, scroll wheel to navigate.

## Features

- **Interactive TUI**: Full-screen interface with live scanning, selection, and deletion
- **Multi-ecosystem**: 60+ ecosystems (Node.js, Rust, Python, Go, Java, and many more)
- **Fast**: Parallel directory scanning and size calculation with rayon
- **Live progress**: Background scan thread streams results as they are found
- **Ecosystem filter**: Popup to show/hide folders by ecosystem
- **Dry-run mode**: Preview what would be deleted without touching disk
- **Mouse support**: Click and scroll in the TUI
- **Flexible filtering**: Target specific ecosystems, exclude directories, depth limits
- **Human-readable sizes**: Automatic unit formatting (B → EB)
- **Plain-text fallback**: `--no-tui` for scripting and piped output

## Architecture

```
src/
├── main.rs              # Binary entry point
├── lib.rs               # run() — branches to TUI or plain-text
├── args.rs              # CLI argument parsing (clap)
├── config/              # Ecosystem loading & pattern matching
├── scanner/             # Directory traversal & parallel size calculation
├── size_util.rs         # Human-readable size formatting
├── deleter.rs           # Folder deletion logic
└── ui/
    ├── ascii.rs         # ASCII art banner selection & rendering
    ├── app.rs           # AppState, ScanEvent, state machine
    ├── tui.rs           # Terminal init/restore, event loop, scan thread
    └── widgets/
        ├── header.rs    # ASCII art banner widget
        ├── list.rs      # Scrollable folder table widget
        ├── footer.rs    # Status bar widget
        └── filter.rs    # Ecosystem filter popup widget
```

## CLI Flags

| Flag | Description | Default |
|------|-------------|---------|
| `-d, --directory <PATH>` | Directory to scan | `.` |
| `-t, --target <LANGS>` | Ecosystems to scan (comma-separated) | All |
| `--all` | Include all ecosystems | `false` |
| `-g, --global` | Include global/user-level caches | `false` |
| `-E, --exclude <DIRS>` | Exclude directories by name | None |
| `-x, --exclude-hidden` | Exclude hidden directories | `false` |
| `--no-recursive` | Current directory only | `false` |
| `--depth <N>` | Maximum directory depth | Unlimited |
| `-f, --full` | Scan from home directory | `false` |
| `-s, --sort <BY>` | Sort by `size` or `path` (plain-text mode) | None |
| `-e, --show-errors` | Show scan error messages | `false` |
| `-D, --delete` | Delete all found folders (plain-text mode) | `false` |
| `--no-tui` | Disable interactive TUI, print plain text | `false` |

## Supported Ecosystems

See [docs/ECOSYSTEMS.md](./docs/ECOSYSTEMS.md) for the full list of 60+ supported ecosystems and their dependency paths.
