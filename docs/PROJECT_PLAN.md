# everykill Implementation Plan

## Overview

Find and delete dependency folders across all ecosystems (Node.js, Rust, Python, Go, Java, etc.).

## Implementation Order

1. ✅ **Ecosystem loading** - Load patterns from `ecosystems/*.json`
2. ✅ **Directory scanning** - Find matching folders
3. ✅ **Size calculation** - Parallel folder sizing
4. ✅ **CLI arguments** - Filtering, sorting, path options
5. ⬜ **Deletion** - Delete folders with confirmation
6. ⬜ **TUI** - Interactive terminal UI with ratatui

## Component Architecture

```
src/
├── main.rs              # Entry point
├── lib.rs               # run() function
├── args.rs             # CLI argument parsing
├── config/
│   └── ecosystem.rs    # Ecosystem loading & matching
├── scanner/
│   ├── dir.rs          # Directory traversal
│   └── size.rs         # Parallel size calculation
├── size_util.rs        # Human-readable size formatting
└── deleter.rs          # Deletion logic (planned)
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
| Clippy linting | ✅ |

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

## Next: Deletion

**File:** `src/deleter.rs`

```rust
pub fn delete_folders(folders: &[PathBuf], dry_run: bool) -> Result<DeleteSummary> {
    // Simple deletion - takes folder list, returns summary
    // No AppState needed - works standalone
}

pub struct DeleteSummary {
    pub deleted_count: usize,
    pub freed_bytes: u64,
    pub errors: Vec<DeleteError>,
}
```

## Then: TUI with AppState

After standalone deletion is working, we build the TUI on top of AppState:

### AppState

```rust
pub struct AppState {
    pub folders: Vec<DiscoveredFolder>,  // All found folders
    pub selected_index: usize,            // Current cursor position
    pub filter_ecosystem: Option<String>, // --lang filter
    pub total_size: u64,                 // Sum of selected sizes
    pub is_scanning: bool,                // Scanning in progress
    pub scan_path: PathBuf,               // Root directory being scanned
}
```

### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│         ____ _  _ ____ ____ _   _ _  _ _ _    _           │
│         |___ |  | |___ |__/  \_/  |_/  | | |    |           │
│         |___  \/  |___ |  \   |   | \_ | |___ |___        │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  [x] node_modules        ~/projects/app/node_modules     │
│  [ ] target/             ~/projects/cli/target           │
│  > vendor/               ~/projects/api/vendor           │
│  [ ] build/              ~/projects/java/build           │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  Selected: 1  |  Total: 1.2 GB                          │
│  [SPACE] select  [ENTER] delete  [Q] quit              │
└──────────────────────────────────────────────────────────┘
```

### Key Events

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| `Space` | Toggle selection |
| `a` | Select all |
| `n` | Deselect all |
| `Enter` | Delete selected |
| `d` | Toggle dry-run mode |
| `f` | Filter by ecosystem |
| `Q` / `Esc` | Quit |

## Testing

| Component | Status |
|-----------|--------|
| Ecosystem loading | ✅ |
| Pattern matching | ✅ |
| Size calculation | ✅ |
| CLI arguments | ✅ |
| Size formatting | ✅ |
| Deletion | Planned |
