# Scanner Module Implementation Plan

## Overview

This plan covers the implementation of the **directory scanner module** (`src/scanner/`), which is responsible for finding dependency folders across all ecosystems.

**Note:** The scanner module can be implemented **before** the TUI/UI components since it has well-defined inputs and outputs and is fully testable in isolation.

## Prerequisites

Before implementing the scanner, we need to complete the following bootstrap work:

### 1. Add Dependencies to `Cargo.toml`

```toml
[dependencies]
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
walkdir = "2.4"
rayon = "1.8"
glob = "0.3"
```

### 2. Create Module Structure

Create the following files:

```
src/
├── config/
│   ├── mod.rs          # Module exports
│   └── ecosystem.rs    # Ecosystem loading & structs
├── scanner/
│   ├── mod.rs          # Module exports
│   ├── dir.rs          # Directory traversal
│   └── size.rs         # Folder size calculation
```

### 3. Define Core Data Structures

These must be created before scanner can compile:

```rust
// src/config/ecosystem.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ecosystem {
    pub name: String,
    pub local: Vec<String>,
    pub global: Vec<String>,
}

impl Ecosystem {
    pub fn matches_folder(&self, folder_name: &str) -> bool {
        // Check if folder_name matches any local or global pattern
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredFolder {
    pub path: PathBuf,
    pub ecosystem: String,
    pub size_bytes: u64,
    pub selected: bool,
}
```

## Implementation Phases

### Phase 1: Ecosystem Loading (`src/config/ecosystem.rs`)

**Responsibility:** Load and parse `ecosystems/*.json` files

**Functions:**
```rust
pub fn load_ecosystems() -> Result<Vec<Ecosystem>> {
    // 1. Glob "ecosystems/*.json"
    // 2. Parse each into Ecosystem struct
    // 3. Return combined list
}

pub fn load_ecosystem(name: &str) -> Result<Ecosystem> {
    // Lazy load: only load specific ecosystem when needed
    // e.g., if user runs `everykill --lang rust`, only load rust.json
}
```

**Key decisions:**
- Cache ecosystems in `HashMap<String, Ecosystem>` for O(1) lookup by name
- Lazy load: only load when needed
- Handle parse errors gracefully (log warning, skip file)

### Phase 2: Directory Scanning (`src/scanner/dir.rs`)

**Responsibility:** Walk directory tree and find matching dependency folders

**Functions:**
```rust
pub fn scan_directory(root: &Path, ecosystems: &[Ecosystem]) -> Vec<DiscoveredFolder> {
    // 1. Walk directory tree (use walkdir)
    // 2. For each entry, check if folder name matches any ecosystem pattern
    // 3. If match, create DiscoveredFolder with path (size = 0 for now)
    // 4. Return all matches
}
```

**Implementation details:**
- Use `walkdir::WalkDir` for directory traversal
- Skip hidden directories (`.git/`, `.svn/`, etc.)
- Skip `node_modules/.cache/` and similar
- Parallel scanning using `rayon` for subtrees
- Skip duplicate folders (same inode) to avoid counting symlinks twice

### Phase 3: Size Calculation (`src/scanner/size.rs`)

**Responsibility:** Calculate folder sizes in parallel

**Functions:**
```rust
pub fn calculate_size(path: &Path) -> Result<u64> {
    // Recursive size calculation using walkdir
    // Sum file sizes (not folder metadata)
}

pub fn calculate_sizes(folders: &mut [DiscoveredFolder]) {
    // For each folder, spawn task to calculate size in parallel
    // Use rayon for parallelization
}
```

**Performance notes:**
- Size calculation is I/O bound - parallelize across CPU cores
- Use `rayon::par_iter()` for simple parallelization

## File Structure After Completion

```
src/
├── config/
│   ├── mod.rs
│   └── ecosystem.rs      # NEW: Ecosystem struct, loading, pattern matching
├── scanner/
│   ├── mod.rs            # NEW: Module exports
│   ├── dir.rs            # NEW: Directory traversal, pattern matching
│   └── size.rs           # NEW: Folder size calculation
└── lib.rs
```

## Testing Strategy

| Component         | Test Approach                                      |
| ----------------- | -------------------------------------------------- |
| Ecosystem loading | Load all JSONs, verify parse, test lazy loading   |
| Pattern matching  | Unit tests with known folder names                 |
| Directory scan    | Create temp dirs, scan, verify matches              |
| Size calculation | Create temp dirs with known sizes, verify accurate |

## Verification After Implementation

After completing the scanner module, we should be able to:

1. Run `cargo test` with unit tests for ecosystem loading and pattern matching
2. Run the application and see it scan the current directory for dependency folders
3. Output a list of discovered folders with their paths and sizes

## What This Enables

Once the scanner module is complete:

- The **CLI arguments** (`--path`, `--lang`, `--all`, etc.) can be added via `clap`
- The **TUI** can be built on top of the scanner to display results interactively
- The **deleter** can be implemented to actually remove folders

The scanner module is the core "business logic" of the application and is independent of how results are displayed or how deletion works.
