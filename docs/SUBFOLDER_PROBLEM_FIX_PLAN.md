# Subfolder Problem Fix Plan

## Problem Statement

When `everykill` scans for dependency folders, it discovers both parent folders and their subfolders. For example, when scanning for Rust's `target/` directory, the scanner finds:

- `./target/` (the actual dependency folder)
- `./target/build/` (a subfolder of target)
- `./target/debug/` (a subfolder of target)
- etc.

This causes:
1. **UI clutter**: Users see redundant entries for what should be a single folder
2. **Confusion**: It's unclear if subfolders should be deleted separately
3. **Double-counting**: Size calculations and deletion could be problematic

## Root Cause Analysis

In `src/scanner/dir.rs`, the `scan_directory()` function uses `walkdir::WalkDir` to traverse directories. The traversal is depth-first, and when it finds a directory matching an ecosystem pattern (e.g., `target/`), it adds it to the results. However, `walkdir` continues to descend into that directory and finds subdirectories like `target/build/`, `target/debug/`, etc.

The current code has:
- `seen_inodes` (line 82) - prevents counting the same inode twice, but doesn't help with parent/child relationships
- `path_contains_skip_dir()` (line 133-134) - filters paths containing skip dirs like `.cache`, but doesn't handle the general subfolder problem

## Solution

**Approach**: Track discovered folder paths and filter out any subsequent paths that are descendants of already-discovered folders.

### Implementation Details

**Location**: `src/scanner/dir.rs`, function `scan_directory()`

**Algorithm**:

1. Add a `HashSet<PathBuf>` called `discovered_prefixes` to track paths of already-discovered folders
2. When a folder is discovered (after all checks pass), add its path to `discovered_prefixes`
3. When considering a new directory entry, check if any discovered path is a prefix of the entry's path
4. If yes, skip adding that entry (it's a subfolder of an already-discovered folder)

**Key insight**: Use `path.starts_with(discovered_path)` to check if a path is a descendant. This correctly handles edge cases:
- `/repo/target/` is a prefix of `/repo/target/debug/` ✓
- `/repo/target/` is NOT a prefix of `/repo/target_extra/` ✓

**Code changes** (in `src/scanner/dir.rs`):

```rust
pub fn scan_directory(
    root: &Path,
    ecosystems: &[Ecosystem],
    include_globals: bool,
    exclude_dirs: &[String],
    exclude_hidden: bool,
    max_depth: Option<usize>,
) -> Vec<DiscoveredFolder> {
    let mut folders = Vec::new();
    let mut seen_inodes: HashSet<u64> = HashSet::new();
    let mut discovered_prefixes: HashSet<PathBuf> = HashSet::new(); // NEW

    // ... existing walker setup ...

    for entry in walker /* ... */ {
        // ... existing checks (file_type, candidates, inode) ...

        // NEW: Check if this path is a subfolder of an already-discovered folder
        let is_subfolder = discovered_prefixes.iter().any(|prefix| {
            entry.path().starts_with(prefix)
        });
        if is_subfolder {
            continue;
        }

        // ... rest of existing logic ...

        // After successfully adding a folder, track its path
        folders.push(DiscoveredFolder::with_resolution(/* ... */));
        discovered_prefixes.insert(entry.path().to_path_buf()); // NEW
    }

    // ... rest of existing filtering ...
}
```

**Important**: The subfolder check must happen AFTER the `seen_inodes` check (to properly handle symlinks) but BEFORE adding to folders.

## Files to Modify

1. `src/scanner/dir.rs` - Add subfolder filtering logic

## Testing Plan

### Unit Tests

1. **Test basic subfolder filtering**: Create directory structure with `target/`, `target/build/`, `target/debug/`. Verify only `target/` is discovered.

2. **Test nested subfolder filtering**: Create `target/debug/deps/`. Verify it's also filtered.

3. **Test non-overlapping names**: Ensure `target/` doesn't filter `target_extra/`.

4. **Test multiple ecosystems**: Ensure filtering works correctly when multiple ecosystems have nested folder structures (e.g., `node_modules/` and `node_modules/.cache/` should filter `.cache` but that's already handled by SKIP_DIRS).

5. **Test with globals**: Ensure global folder discovery also respects subfolder filtering.

### Test Cases to Add

```rust
#[test]
fn test_scan_filters_target_subfolders() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create: target/, target/build/, target/debug/
    fs::create_dir_all(project_dir.join("target/build")).unwrap();
    fs::create_dir_all(project_dir.join("target/debug")).unwrap();

    let ecosystems = vec![Ecosystem {
        name: "Rust".to_string(),
        local: vec!["target/".to_string()],
        global: vec![],
        markers: vec![],
    }];

    let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

    // Should only find target/, not its subfolders
    assert_eq!(folders.len(), 1);
    assert!(folders[0].path.ends_with("target"));
}

#[test]
fn test_scan_does_not_filter_similar_names() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    fs::create_dir_all(project_dir.join("target")).unwrap();
    fs::create_dir_all(project_dir.join("target_extra")).unwrap();

    let ecosystems = vec![Ecosystem {
        name: "Rust".to_string(),
        local: vec!["target/".to_string()],
        global: vec![],
        markers: vec![],
    }];

    let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

    // Should find both: target/ and target_extra/
    assert_eq!(folders.len(), 2);
}

#[test]
fn test_scan_filters_deeply_nested_subfolders() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    fs::create_dir_all(project_dir.join("target/debug/deps/foo")).unwrap();

    let ecosystems = vec![Ecosystem {
        name: "Rust".to_string(),
        local: vec!["target/".to_string()],
        global: vec![],
        markers: vec![],
    }];

    let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

    assert_eq!(folders.len(), 1);
    assert!(folders[0].path.ends_with("target"));
}
```

## Edge Cases to Consider

1. **Symlinks**: If `target/` is a symlink to `/somewhere/else/target`, and there are subfolders, the current inode tracking should handle this via `seen_inodes`.

2. **Depth limits**: If `--depth` is specified, the walkdir depth limit should still work correctly. The subfolder filtering happens at the application level, not the walkdir level.

3. **Exclude directories**: The existing exclude logic should work as-is. A folder excluded by name won't be discovered, so its subfolders also won't be discovered (because they won't be visited by walkdir if the parent is excluded via `filter_entry`).

4. **Hidden directories**: The existing `--exclude-hidden` logic should continue to work.

## Alternative Approaches Considered

### Approach 2: Stop walkdir descent into matched folders

Instead of filtering after the fact, we could use walkdir's `filter_entry` to not descend into already-matched directories. However, walkdir doesn't provide a direct way to "don't descend into this directory" from within `filter_entry`. We would need to use `WalkDir::into_iter()` with manual state management.

**Verdict**: More complex, not worth the benefit.

### Approach 3: Post-scan filtering

After scanning, filter the `folders` vector to remove entries that are subfolders of other entries.

**Verdict**: Simpler but less efficient (wastes work scanning subfolders). Also requires sorting or more complex comparison.

## Summary

The fix is straightforward:
1. Track paths of discovered folders in a `HashSet<PathBuf>`
2. Before adding a new folder, check if it's a descendant of any already-discovered folder
3. If yes, skip it

This ensures that when `target/` is discovered, `target/build/`, `target/debug/`, and any other nested subfolders are automatically filtered out.
