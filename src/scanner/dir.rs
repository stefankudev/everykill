use crate::config::{DiscoveredFolder, Ecosystem};
use std::collections::HashSet;
use std::path::Path;

const SKIP_DIRS: &[&str] = &[".git", ".svn", ".hg", "node_modules/.cache", ".cache"];

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

                    for ecosystem in ecosystems {
                        if ecosystem.matches_folder_with_globals(&file_name, include_globals) {
                            if let Ok(inode) = get_inode(entry.path()) {
                                if seen_inodes.contains(&inode) {
                                    continue;
                                }
                                seen_inodes.insert(inode);
                            }

                            folders.push(DiscoveredFolder::new(
                                entry.path().to_path_buf(),
                                ecosystem.name.clone(),
                            ));
                            break;
                        }
                    }
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
