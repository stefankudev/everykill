# everykill Implementation Plan

## Overview

Find and delete dependency folders across all ecosystems (Node.js, Rust, Python, Go, Java, etc.).

## Implementation Order

1. ✅ **Ecosystem loading** - Load patterns from `ecosystems/*.json`
2. ✅ **Directory scanning** - Find matching folders
3. ✅ **Size calculation** - Parallel folder sizing
4. ✅ **CLI arguments** - Filtering, sorting, path options
5. ✅ **Deletion** - Delete folders with confirmation
6. ✅ **TUI** - Interactive terminal UI with ratatui

## Component Architecture

```
src/
├── main.rs              # Entry point
├── lib.rs               # run() → TUI or plain-text
├── args.rs              # CLI argument parsing
├── config/
│   └── ecosystem.rs     # Ecosystem loading & matching
├── scanner/
│   ├── dir.rs           # Directory traversal
│   └── size.rs          # Parallel size calculation
├── size_util.rs         # Human-readable size formatting
├── deleter.rs           # Deletion logic
└── ui/
    ├── ascii.rs         # ASCII art selection & rendering
    ├── app.rs           # AppState, ScanEvent, state machine
    ├── tui.rs           # Terminal init/restore, event loop, scan thread
    └── widgets/
        ├── header.rs    # ASCII art banner widget
        ├── list.rs      # Scrollable folder table widget
        ├── footer.rs    # Status bar widget
        └── filter.rs    # Ecosystem filter popup widget
```

## Implemented Features

| Feature | Status |
|---------|--------|
| 60+ ecosystem patterns | ✅ |
| Parallel scanning (rayon) | ✅ |
| Inode deduplication | ✅ |
| CLI filtering (`-t`, `-E`, `-x`) | ✅ |
| Depth control (`--depth`, `--no-recursive`) | ✅ |
| Sort by size/path (`-s`) | ✅ |
| Human-readable sizes (B → EB) | ✅ |
| CLI deletion (`-D, --delete`) | ✅ |
| Clippy linting | ✅ |
| Interactive TUI (ratatui) | ✅ |
| Background scan thread (mpsc) | ✅ |
| Ecosystem filter popup | ✅ |
| Deletion confirmation prompt | ✅ |
| Mouse support (click + scroll) | ✅ |
| `--no-tui` plain-text fallback | ✅ |

## CLI Flags

| Flag | Description | Default |
| ---- |-------------|---------|
| `-d, --directory <PATH>` | Directory to scan | `.` |
| `-t, --target <LANGS>` | Ecosystems to scan | All local |
| `--all` | Include all ecosystems | `false` |
| `-g, --global` | Include global caches | `false` |
| `-E, --exclude <DIRS>` | Exclude directories | None |
| `-x, --exclude-hidden` | Exclude hidden dirs | `false` |
| `--no-recursive` | Current dir only | `false` |
| `--depth <N>` | Max depth | Unlimited |
| `-f, --full` | Scan from home | `false` |
| `-s, --sort <BY>` | Sort by size/path | None |
| `-e, --show-errors` | Show errors | `false` |
| `-D, --delete` | Delete found folders | `false` |
| `--no-tui` | Disable interactive TUI | `false` |

## TUI Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate list |
| `Page Up` / `Page Down` | Scroll by page |
| `Home` / `End` | Jump to top / bottom |
| `Space` | Toggle selection |
| `a` | Select all visible |
| `n` | Deselect all |
| `d` | Toggle dry-run |
| `f` | Open ecosystem filter popup |
| `Enter` | Delete selected (with confirmation) |
| `q` / `Q` / `Esc` | Quit |

## Testing

| Component | Status |
|-----------|--------|
| Ecosystem loading | ✅ |
| Pattern matching | ✅ |
| Size calculation | ✅ |
| CLI arguments | ✅ |
| Size formatting | ✅ |
| Deletion | ✅ |
| AppState / ScanEvent | ✅ |
