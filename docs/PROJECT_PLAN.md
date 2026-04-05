# everykill Implementation Plan

## Overview

The core functionality consists of:
1. **Load ecosystems** from `ecosystems/*.json`
2. **Scan directories** to find matching dependency folders
3. **Calculate sizes** of found folders
4. **Display interactive TUI** with folder list
5. **Delete selected folders** with confirmation

## Component Architecture

```
src/
├── main.rs                  # Entry point, clap args
├── lib.rs                   # run() function
├── config/
│   └── mod.rs
│   └── ecosystem.rs         # Ecosystem loading & parsing
├── scanner/
│   ├── mod.rs
│   ├── dir.rs               # Directory traversal
│   └── size.rs              # Folder size calculation
├── ui/
│   ├── mod.rs
│   ├── app.rs               # Main app state & event loop
│   ├── widgets.rs           # List, status bar, etc.
│   └── style.rs             # Colors/theme
└── deleter.rs              # Deletion with confirmation
```

## Data Structures

### Ecosystem
```rust
pub struct Ecosystem {
    pub name: String,        // "Node.js", "Rust", etc.
    pub local: Vec<String>, // Folder patterns: "node_modules/", "target/"
    pub global: Vec<String>, // Cache paths: "~/.npm/", etc.
}
```

### DiscoveredFolder
```rust
pub struct DiscoveredFolder {
    pub path: PathBuf,           // Full path to folder
    pub ecosystem: String,       // Which ecosystem matched
    pub size_bytes: u64,          // Calculated size
    pub selected: bool,          // User selection state
}
```

### AppState
```rust
pub struct AppState {
    pub folders: Vec<DiscoveredFolder>,  // All found folders
    pub selected_index: usize,            // Current cursor position
    pub filter_ecosystem: Option<String>, // --lang filter
    pub total_size: u64,                   // Sum of selected sizes
    pub is_scanning: bool,                  // Scanning in progress
    pub scan_path: PathBuf,                // Root directory being scanned
}
```

## Implementation Phases

### ✅ Phase 1: Ecosystem Loading (IMPLEMENTED)

**File:** `src/config/ecosystem.rs`

```rust
pub fn load_ecosystems() -> Result<Vec<Ecosystem>> {
    // 1. Glob "ecosystems/*.json"
    // 2. Parse each into Ecosystem struct
    // 3. Return combined list
}

impl Ecosystem {
    pub fn matches_folder(&self, folder_name: &str) -> bool {
        // Check if folder_name matches any local or global pattern
    }
}
```

**Key decisions:**
- Cache ecosystems in `HashMap<String, Ecosystem>` for O(1) lookup by name
- Lazy load: only load when needed (e.g., `--lang rust` loads only rust.json)

### ✅ Phase 2: Directory Scanning (IMPLEMENTED)

**File:** `src/scanner/dir.rs`

```rust
pub fn scan_directory(root: &Path, ecosystems: &[Ecosystem]) -> Vec<DiscoveredFolder> {
    // 1. Walk directory tree (use walkdir or std::fs::ReadDir)
    // 2. For each entry, check if folder name matches any ecosystem pattern
    // 3. If match, create DiscoveredFolder with path
    // 4. Return all matches
}
```

**Performance considerations:**
- **Parallel scanning**: Use rayon or tokio to scan subtrees in parallel
- **Skip hidden dirs**: Skip `.git/`, `.svn/`, etc. by default
- **Depth limit**: Optional `--max-depth` flag
- **I/O optimization**: Use `ReadDir` buffering, batch size calculations

**Key decisions:**
- Parallel scanning implemented immediately with rayon
- Skip common non-project dirs: `.git/`, `node_modules/.cache/`, etc.
- Don't calculate sizes during scan - just collect paths first
- Skip duplicate folders (same inode) to avoid counting symlinked directories twice

### ✅ Phase 3: Size Calculation (IMPLEMENTED)

**File:** `src/scanner/size.rs`

```rust
pub fn calculate_size(path: &Path) -> Result<u64> {
    // Recursive size calculation
    // Use walkdir to iterate all files
    // Sum file sizes (not folder metadata)
}

pub fn calculate_sizes/folders: &mut [DiscoveredFolder]) {
    // For each folder, spawn task to calculate size
    // Use rayon for parallelization
}
```

**Performance considerations:**
- Size calculation is I/O bound - parallelize across CPU cores
- Use `rayon` for simple parallel iterators
- Show progress for large folders (>1GB estimated)

### Phase 4: TUI with Ratatui

**File:** `src/ui/app.rs`

```rust
pub fn run(state: &mut AppState) {
    // 1. Setup terminal: enable raw mode, hide cursor
    // 2. Create ratatui Terminal
    // 3. Event loop:
    //    - poll for events (key presses, window resize)
    //    - match event: ArrowUp/Down (move cursor), Space (toggle select), Enter (delete), Q (quit)
    //    - render: clear screen, draw list, draw status bar
    // 4. Cleanup: restore terminal state
}
```

**UI Layout:**
```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│         _______ _    _ _______  ______                   │
│         |______  \  /  |______ |_____/                   │
│         |______   \/   |______ |    \_                   │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  [x] node_modules        ~/projects/app/node_modules     │
│  [ ] target/             ~/projects/cli/target           │
│  > vendor/               ~/projects/api/vendor           │
│  [ ] build/              ~/projects/java/build            │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  Selected: 1  |  Total: 1.2 GB                          │
│  [SPACE] select  [ENTER] delete  [Q] quit              │
└──────────────────────────────────────────────────────────┘
```

**Key events:**
| Key                 | Action                              |
| ------------------- | ----------------------------------- |
| `↑` / `↓`           | Move cursor                         |
| `↑` / `↓` + `Shift` | Scroll by page                      |
| `Space`             | Toggle selection on current item    |
| `a`                 | Select all                          |
| `n`                 | Deselect all                        |
| `Enter`             | Delete selected (with confirmation) |
| `d`                 | Toggle dry-run mode                 |
| `f`                 | Filter by ecosystem                 |
| `Q` / `Esc`         | Quit                                |

### Phase 5: Deletion with Confirmation

**File:** `src/deleter.rs`

```rust
pub fn delete_folders(folders: &[PathBuf], dry_run: bool) -> Result<DeleteSummary> {
    if dry_run {
        // Just show what would be deleted
        // Return summary without actually deleting
    } else {
        // Show confirmation dialog
        // If confirmed, delete each folder
        // Return summary of what was deleted
    }
}

pub struct DeleteSummary {
    pub deleted_count: usize,
    pub freed_bytes: u64,
    pub errors: Vec<DeleteError>,
}
```

**Error handling:**
- Permission denied → skip and log error
- Folder already deleted → ignore
- Symbolic link → only delete symlink, not target
- Show errors in summary after deletion

## CLI Arguments (clap) - IMPLEMENTED

**File:** `src/args.rs`

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Directory to scan
    #[arg(short = 'd', long = "directory", default_value = ".")]
    path: PathBuf,

    /// Ecosystems to scan (comma-separated)
    #[arg(short = 't', long = "target")]
    target: Option<String>,

    /// Include all ecosystems
    #[arg(long = "all")]
    all: bool,

    /// Include global/user-level caches
    #[arg(short = 'g', long = "global")]
    global: bool,

    /// Exclude directories by name (comma-separated)
    #[arg(short = 'E', long = "exclude")]
    exclude: Option<String>,

    /// Exclude hidden directories
    #[arg(short = 'x', long = "exclude-hidden")]
    exclude_hidden: bool,

    /// Don't scan subdirectories
    #[arg(long = "no-recursive")]
    no_recursive: bool,

    /// Maximum directory depth
    #[arg(long = "depth")]
    depth: Option<usize>,

    /// Scan from home directory
    #[arg(short = 'f', long = "full")]
    full: bool,

    /// Sort results by size or path
    #[arg(short = 's', long = "sort")]
    sort: Option<SortBy>,

    /// Show error messages
    #[arg(short = 'e', long = "show-errors")]
    show_errors: bool,
}
```

**CLI Flags Table:**

| Arg | Description | Default |
|-----|-------------|---------|
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

## Error Handling Strategy

| Error                       | Handling                         |
| --------------------------- | -------------------------------- |
| JSON parse error            | Log warning, skip that ecosystem |
| Permission denied on scan   | Log warning, skip that directory |
| Permission denied on delete | Log error, continue with others  |
| Folder not found on delete  | Log info, continue               |
| Broken symlink              | Log warning, skip                |
| Disk I/O error              | Log error, abort scan            |

All errors should be:
1. Logged to stderr
2. Collected in a summary
3. Presented to user at end of operation

## File Organization Summary

| File | Responsibility | Status |
|------|---------------|--------|
| `src/args.rs` | CLI argument parsing | ✅ Done |
| `src/config/ecosystem.rs` | Load & parse `ecosystems/*.json` | ✅ Done |
| `src/scanner/mod.rs` | Module exports | ✅ Done |
| `src/scanner/dir.rs` | Walk directories, find matches | ✅ Done |
| `src/scanner/size.rs` | Calculate folder sizes | ✅ Done |
| `src/size_util.rs` | Size formatting utility | ✅ Done |
| `src/ui/app.rs` | Main TUI event loop | Planned |
| `src/ui/widgets.rs` | List widget, status bar | Planned |
| `src/deleter.rs` | Delete folders, confirmation | Planned |

## Testing Strategy

| Component | Test Approach | Status |
| --------- | ------------ | ------ |
| Ecosystem loading | Load all JSONs, verify parse | ✅ Done |
| Pattern matching | Unit tests with known folder names | ✅ Done |
| Size calculation | Use temp dirs with known sizes | ✅ Done |
| CLI arguments | Unit tests for arg parsing | ✅ Done |
| Size formatting | Unit tests for all units/precisions | ✅ Done |
| Deletion | Create temp dirs, delete, verify removal | Planned |
| UI | Manual testing only (ratatui is hard to test) | Planned |

## Decisions

| Decision | Details | Status |
| -------- | -------- | ------ |
| Default scanning | Scans current directory and all subfolders | ✅ Done |
| `--global` / `-g` flag | Enables scanning of user-level caches | ✅ Done |
| Parallel scanning | Implemented with rayon | ✅ Done |
| Delete confirmation | **Per-item** (npkill-style) | Planned |
| Sort order | User-controlled via `-s` flag | ✅ Done |
| Duplicate detection | Skip if same inode found (handles symlinks) | ✅ Done |
| Size formatting | Dynamic precision based on magnitude | ✅ Done |
