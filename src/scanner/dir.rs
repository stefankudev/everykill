use crate::config::{Confidence, DiscoveredFolder, Ecosystem};
use std::collections::HashSet;
use std::path::Path;

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

    let mut walker = walkdir::WalkDir::new(root).follow_links(false);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    for entry in walker
        .into_iter()
        .filter_entry(|e| !should_skip_entry(e, exclude_dirs, exclude_hidden))
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

                    if let Ok(inode) = get_inode(entry.path()) {
                        if seen_inodes.contains(&inode) {
                            continue;
                        }
                        seen_inodes.insert(inode);
                    }

                    let project_root = entry.path().parent().unwrap_or(entry.path());
                    let resolved = resolve_ecosystem(&candidates, project_root);

                    folders.push(DiscoveredFolder::with_resolution(
                        entry.path().to_path_buf(),
                        resolved.label,
                        resolved.confidence,
                    ));
                }
            }
            Err(e) => {
                eprintln!("Warning: error accessing entry: {}", e);
            }
        }
    }

    folders
        .into_iter()
        .filter(|f| !path_contains_skip_dir(&f.path))
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

    if exclude_dirs.iter().any(|d| name_str == *d) {
        return true;
    }

    SKIP_DIRS.contains(&name_str.as_ref())
}

fn path_contains_skip_dir(path: &Path) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(name) = c {
            should_skip(name)
        } else {
            false
        }
    })
}

fn should_skip(name: &std::ffi::OsStr) -> bool {
    let name_str = name.to_string_lossy();
    SKIP_DIRS.iter().any(|s| name_str == *s)
}

fn get_inode(path: &Path) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.ino())
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
}
