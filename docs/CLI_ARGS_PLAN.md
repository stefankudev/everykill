# CLI Arguments Implementation Plan

## Overview

Add command-line argument parsing using `clap` to make everykill scriptable and usable without a TUI. Flags are designed to be familiar to npkill users while supporting all ecosystems.

## Flag Specification

### CLI Flags Table

| Arg | Description | Default |
|-----|-------------|---------|
| `-d, --directory <PATH>` | Directory to scan | `.` (current directory) |
| `-t, --target <LANGS>` | Ecosystems to scan (comma-separated) | All local |
| `--all` | Include all ecosystems | `false` |
| `-g, --global` | Include global/user-level caches | `false` |
| `-E, --exclude <DIRS>` | Exclude directories by name (comma-separated) | None |
| `-x, --exclude-hidden` | Exclude hidden directories (dot directories) | `false` |
| `--no-recursive` | Don't scan subdirectories (alias for `--depth 0`) | `false` |
| `--depth <N>` | Maximum directory depth (0 = current only) | Unlimited |
| `-f, --full` | Scan from home directory | `false` |
| `-s, --sort <BY>` | Sort by `size` or `path` | None (insertion order) |
| `-e, --show-errors` | Show error messages | `false` |
| `-h, --help` | Show help | - |
| `-v, --version` | Show version | - |

### Notes on Flag Design

- **`-t, --target`**: Follows npkill's muscle memory. Accepts comma-separated ecosystem names matching filenames in `ecosystems/*.json` (e.g., `-t nodejs,rust,python`)
- **`--all`**: Enables all ecosystems. When combined with `--global`, includes all local AND global caches
- **`-g, --global`**: Only meaningful when `--all` is set, or when specific ecosystems are selected with `-t`
- **`--no-recursive`**: Alias for `--depth 0`
- **`-E, --exclude`**: Excludes directories by name anywhere in the scan path (not by ecosystem)

---

## Implementation Design

### Dependencies

Add to `Cargo.toml`:
```toml
clap = { version = "4.5", features = ["derive"] }
```

### New Module Structure

```
src/
├── args.rs              # NEW: CLI argument parsing
├── lib.rs               # Update to use args
└── ...
```

### Data Structures

```rust
// src/args.rs

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "Find and remove dependency folders across all ecosystems")]
pub struct Args {
    /// Directory to scan
    #[arg(short = 'd', long = "directory", default_value = ".")]
    pub path: PathBuf,

    /// Ecosystems to scan (comma-separated, e.g., "nodejs,rust,python")
    #[arg(short = 't', long = "target")]
    pub target: Option<String>,

    /// Include all ecosystems (local patterns only, unless --global is also used)
    #[arg(long = "all")]
    pub all: bool,

    /// Include global/user-level caches
    #[arg(short = 'g', long = "global")]
    pub global: bool,

    /// Exclude directories by name (comma-separated)
    #[arg(short = 'E', long = "exclude")]
    pub exclude: Option<String>,

    /// Exclude hidden directories
    #[arg(short = 'x', long = "exclude-hidden")]
    pub exclude_hidden: bool,

    /// Don't scan subdirectories (equivalent to --depth 0)
    #[arg(long = "no-recursive")]
    pub no_recursive: bool,

    /// Maximum directory depth (0 = current directory only)
    #[arg(long = "depth")]
    pub depth: Option<usize>,

    /// Scan from home directory
    #[arg(short = 'f', long = "full")]
    pub full: bool,

    /// Sort results by size or path
    #[arg(short = 's', long = "sort")]
    pub sort: Option<SortBy>,

    /// Show error messages
    #[arg(short = 'e', long = "show-errors")]
    pub show_errors: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    Size,
    Path,
}

impl Args {
    /// Get scan path (resolves ~ and handles --full)
    pub fn get_scan_path(&self) -> PathBuf {
        if self.full {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        } else {
            self.path.clone()
        }
    }

    /// Get depth limit (None = unlimited)
    pub fn get_depth_limit(&self) -> Option<usize> {
        if self.no_recursive {
            Some(0)
        } else {
            self.depth
        }
    }

    /// Get ecosystems to scan
    pub fn get_ecosystems(&self, all_ecosystems: &[Ecosystem]) -> Vec<Ecosystem> {
        if self.all {
            all_ecosystems.to_vec()
        } else if let Some(targets) = &self.target {
            let target_names: Vec<&str> = targets.split(',').map(|s| s.trim()).collect();
            all_ecosystems
                .iter()
                .filter(|e| target_names.contains(&e.name.to_lowercase()))
                .cloned()
                .collect()
        } else {
            // Default: all local patterns only
            all_ecosystems.to_vec()
        }
    }

    /// Check if we should include global patterns for an ecosystem
    pub fn should_include_globals(&self) -> bool {
        self.global || self.all
    }

    /// Get excluded directory names
    pub fn get_excluded_dirs(&self) -> Vec<String> {
        self.exclude
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
}
```

---

## Integration Points

### Update `lib.rs`

```rust
pub mod args;

use args::Args;
use clap::Parser;

pub fn run() {
    let args = Args::parse();

    // Load ecosystems
    let all_ecosystems = config::load_ecosystems()
        .expect("failed to load ecosystems");

    let ecosystems = args.get_ecosystems(&all_ecosystems);
    let include_globals = args.should_include_globals();

    // Build exclusion list
    let excluded_dirs = args.get_excluded_dirs();
    let exclude_hidden = args.exclude_hidden;
    let depth_limit = args.get_depth_limit();

    // Scan
    let folders = scanner::scan_directory(
        &args.get_scan_path(),
        &ecosystems,
        include_globals,
        &excluded_dirs,
        exclude_hidden,
        depth_limit,
    );

    // Sort if requested
    let folders = match args.sort {
        Some(SortBy::Size) => {
            let mut folders = folders;
            folders.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            folders
        }
        Some(SortBy::Path) => {
            let mut folders = folders;
            folders.sort_by(|a, b| a.path.cmp(&b.path));
            folders
        }
        None => folders,
    };

    // Calculate sizes
    let mut folders = folders;
    scanner::calculate_sizes(&mut folders);

    // Output
    for folder in &folders {
        println!(
            "  {} ({}) - {}",
            folder.path.display(),
            folder.ecosystem,
            format_size(folder.size_bytes)
        );
    }
}
```

### Update `scanner::scan_directory` Signature

```rust
// src/scanner/dir.rs

pub fn scan_directory(
    root: &Path,
    ecosystems: &[Ecosystem],
    include_globals: bool,
    exclude_dirs: &[String],
    exclude_hidden: bool,
    max_depth: Option<usize>,
) -> Vec<DiscoveredFolder> {
    // Implementation updated to handle new parameters
}
```

---

## Flag Processing Flow

```
user runs: everykill -t nodejs,rust -E "target,vendor"
                              │
                              ▼
clap parses:                                    │
  target = "nodejs,rust"                       │
  exclude = "target,vendor"                    │
  path = "."                                   │
                              │                 │
                              ▼                 ▼
Args::get_ecosystems() ──► filters to Node.js and Rust only
Args::get_excluded_dirs() ──► returns ["target", "vendor"]
Args::should_include_globals() ──► returns false
                              │
                              ▼
scanner::scan_directory() receives:
  ecosystems = [Node.js, Rust]
  exclude_dirs = ["target", "vendor"]
  exclude_hidden = false
  max_depth = None (unlimited)
```

---

## Testing Strategy

| Test | Description |
|------|-------------|
| `test_parse_target_flag` | Parse `-t nodejs,rust` correctly |
| `test_parse_exclude_flag` | Parse `-E "target,vendor"` correctly |
| `test_no_recursive_alias` | `--no-recursive` sets depth to 0 |
| `test_full_flag` | `-f` resolves to home directory |
| `test_get_ecosystems_all` | `--all` returns all ecosystems |
| `test_get_ecosystems_filtered` | `-t rust` returns only Rust |
| `test_get_excluded_dirs` | Correctly splits comma-separated list |
| `test_sort_by_size` | Verify sorting works correctly |
| `test_sort_by_path` | Verify sorting works correctly |

---

## Usage Examples

```bash
# Scan current directory for all dependency folders
everykill

# Scan specific directory
everykill -d ~/projects

# Target specific ecosystems
everykill -t nodejs,rust

# Include global caches
everykill -t nodejs --global

# Include all ecosystems with globals
everykill --all --global

# Exclude specific directories
everykill -E "target,vendor,.next"

# Don't scan subdirectories
everykill --no-recursive

# Limit depth to 2 levels
everykill --depth 2

# Sort by size (largest first)
everykill -s size

# Sort by path
everykill -s path

# Full home directory scan
everykill -f

# Combine options
everykill -f -t nodejs,python -E "node_modules/.cache" -s size
```

---

## File Changes Summary

| File | Change |
|------|--------|
| `Cargo.toml` | Add `clap` dependency |
| `src/args.rs` | NEW - CLI argument parsing |
| `src/lib.rs` | Update to use `Args` and new scan signature |
| `src/scanner/dir.rs` | Update `scan_directory` signature with new params |
| `src/scanner/mod.rs` | Update exports |

---

## Dependencies Added

```toml
clap = { version = "4.5", features = ["derive"] }
```

---

## Implementation Order

1. Add `clap` to `Cargo.toml`
2. Create `src/args.rs` with `Args` struct and helper methods
3. Update `src/scanner/dir.rs` to accept new parameters
4. Update `src/lib.rs` to parse args and pass to scanner
5. Add unit tests for `args.rs`
6. Verify with `cargo test` and `cargo clippy`
7. Test manually with various flag combinations
