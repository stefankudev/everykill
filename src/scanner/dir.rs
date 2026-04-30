use crate::config::{Confidence, DiscoveredFolder, Ecosystem};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[".git", ".svn", ".hg", "node_modules/.cache", ".cache"];

// ---------------------------------------------------------------------------
// Confidence scoring
// ---------------------------------------------------------------------------

/// Result of resolving a discovered folder to an ecosystem.
#[derive(Debug, Clone)]
pub struct ResolvedEcosystem {
    /// Display label, e.g. "Rust" or "Rust / Clojure".
    pub label: String,
    pub confidence: Confidence,
}

/// Look at all `candidates` that matched this folder and return the best
/// interpretation based on marker files present in `project_root`.
fn resolve_ecosystem(candidates: &[&Ecosystem], project_root: &Path) -> ResolvedEcosystem {
    debug_assert!(!candidates.is_empty());

    if candidates.len() == 1 {
        return ResolvedEcosystem {
            label: candidates[0].name.clone(),
            confidence: Confidence::Certain,
        };
    }

    let mut confirmed: Vec<&Ecosystem> = Vec::new();

    for eco in candidates {
        if eco.markers.is_empty() {
            continue;
        }
        let all_present = eco.markers.iter().all(|m| project_root.join(m).is_file());
        if all_present {
            confirmed.push(eco);
        }
    }

    if confirmed.len() == 1 {
        ResolvedEcosystem {
            label: confirmed[0].name.clone(),
            confidence: Confidence::Confirmed,
        }
    } else if confirmed.is_empty() {
        ResolvedEcosystem {
            label: candidates
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>()
                .join(" / "),
            confidence: Confidence::Undetected,
        }
    } else {
        ResolvedEcosystem {
            label: confirmed
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>()
                .join(" / "),
            confidence: Confidence::Ambiguous,
        }
    }
}

// ---------------------------------------------------------------------------
// Directory scanning
// ---------------------------------------------------------------------------

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
    let mut discovered_prefixes: HashSet<PathBuf> = HashSet::new();

    let mut walker = walkdir::WalkDir::new(root).follow_links(false);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    let mut all_exclude_dirs: Vec<String> = exclude_dirs.to_vec();
    for &skip in SKIP_DIRS {
        if !all_exclude_dirs.contains(&skip.to_string()) {
            all_exclude_dirs.push(skip.to_string());
        }
    }

    for entry in walker
        .into_iter()
        .filter_entry(|e| !should_skip_entry(e, &all_exclude_dirs, exclude_hidden))
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_dir() {
                    let file_name = entry.file_name().to_string_lossy();

                    let candidates: Vec<&Ecosystem> = ecosystems
                        .iter()
                        .filter(|e| e.matches_folder_with_globals(&file_name, include_globals))
                        .collect();

                    if candidates.is_empty() {
                        continue;
                    }

                    if let Some(id) = get_unique_id(entry.path()) {
                        if seen_inodes.contains(&id) {
                            continue;
                        }
                        seen_inodes.insert(id);
                    }

                    let entry_path = entry.path().to_path_buf();
                    if discovered_prefixes
                        .iter()
                        .any(|prefix| entry_path.starts_with(prefix))
                    {
                        continue;
                    }

                    let project_root = entry_path.parent().unwrap_or(&entry_path);
                    let resolved = resolve_ecosystem(&candidates, project_root);

                    folders.push(DiscoveredFolder::with_resolution(
                        entry_path.clone(),
                        resolved.label,
                        resolved.confidence,
                    ));
                    discovered_prefixes.insert(entry_path);
                }
            }
            Err(e) => {
                eprintln!("Warning: error accessing entry: {}", e);
            }
        }
    }

    folders
        .into_iter()
        .filter(|f| !path_contains_skip_dir(&f.path, &all_exclude_dirs))
        .collect()
}

fn should_skip_entry(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[String],
    exclude_hidden: bool,
) -> bool {
    let name = entry.file_name();
    let name_str = name.to_string_lossy();

    if exclude_hidden && name_str.starts_with('.') {
        return true;
    }

    exclude_dirs.iter().any(|d| name_str == *d)
}

fn path_contains_skip_dir(path: &Path, exclude_dirs: &[String]) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(name) = c {
            let name_str = name.to_string_lossy();
            exclude_dirs.iter().any(|d| name_str == *d)
        } else {
            false
        }
    })
}

#[cfg(unix)]
fn get_unique_id(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).ok().map(|m| m.ino())
}

#[cfg(not(unix))]
fn get_unique_id(_path: &Path) -> Option<u64> {
    // On non-Unix platforms, we don't have a stable way to get a unique ID
    // without unstable features or extra dependencies. Since we don't follow
    // links, path-based uniqueness is sufficient.
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_finds_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join("some/path/node_modules")).unwrap();
        fs::write(project_dir.join("some/path/node_modules/.gitkeep"), "").unwrap();

        let ecosystems = vec![Ecosystem {
            name: "Node.js".to_string(),
            local: vec!["node_modules/".to_string()],
            global: vec![],
            markers: vec![],
        }];

        let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

        assert!(!folders.is_empty());
        assert!(folders.iter().any(|f| f.path.ends_with("node_modules")));
        assert_eq!(folders[0].ecosystem, "Node.js");
    }

    #[test]
    fn test_scan_skips_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join(".git/node_modules")).unwrap();
        fs::create_dir_all(project_dir.join("node_modules")).unwrap();

        let ecosystems = vec![Ecosystem {
            name: "Node.js".to_string(),
            local: vec!["node_modules/".to_string()],
            global: vec![],
            markers: vec![],
        }];

        let folders = scan_directory(project_dir, &ecosystems, true, &[], true, None);

        assert!(folders
            .iter()
            .all(|f| !f.path.to_string_lossy().contains(".git")));
    }

    #[test]
    fn test_scan_skips_node_modules_cache() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join("node_modules/.cache")).unwrap();

        let ecosystems = vec![Ecosystem {
            name: "Node.js".to_string(),
            local: vec!["node_modules/".to_string()],
            global: vec![],
            markers: vec![],
        }];

        let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

        assert_eq!(folders.len(), 1);
        assert!(folders[0].path.ends_with("node_modules"));
        assert!(!folders
            .iter()
            .any(|f| f.path.to_string_lossy().contains(".cache")));
    }

    #[test]
    fn test_scan_filters_target_subfolders() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join("target/build")).unwrap();
        fs::create_dir_all(project_dir.join("target/debug")).unwrap();

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

    #[test]
    fn test_scan_does_not_filter_similar_names() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join("vendor")).unwrap();
        fs::create_dir_all(project_dir.join("target")).unwrap();

        let ecosystems = vec![Ecosystem {
            name: "Go".to_string(),
            local: vec!["vendor/".to_string(), "target/".to_string()],
            global: vec![],
            markers: vec![],
        }];

        let folders = scan_directory(project_dir, &ecosystems, true, &[], false, None);

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
}
