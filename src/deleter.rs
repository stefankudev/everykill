use crate::config::DiscoveredFolder;
use crate::size_util::format_size;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeleteError {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct DeleteSummary {
    pub deleted_count: usize,
    pub freed_bytes: u64,
    pub errors: Vec<DeleteError>,
}

impl DeleteSummary {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn delete_folders(folders: &[DiscoveredFolder], dry_run: bool) -> DeleteSummary {
    let mut summary = DeleteSummary::new();

    for folder in folders {
        if !folder.selected {
            continue;
        }

        if dry_run {
            println!(
                "[DRY-RUN] Would delete: {} ({} - {})",
                folder.path.display(),
                folder.ecosystem,
                format_size(folder.size_bytes)
            );
            summary.deleted_count += 1;
            summary.freed_bytes += folder.size_bytes;
        } else {
            match std::fs::remove_dir_all(&folder.path) {
                Ok(_) => {
                    println!(
                        "Deleted: {} ({} - {})",
                        folder.path.display(),
                        folder.ecosystem,
                        format_size(folder.size_bytes)
                    );
                    summary.deleted_count += 1;
                    summary.freed_bytes += folder.size_bytes;
                }
                Err(e) => {
                    let error = DeleteError {
                        path: folder.path.clone(),
                        reason: e.to_string(),
                    };
                    summary.errors.push(error);
                }
            }
        }
    }

    summary
}

pub fn print_delete_summary(summary: &DeleteSummary) {
    println!();
    if summary.errors.is_empty() {
        println!(
            "Deleted {} folders ({} freed)",
            summary.deleted_count,
            format_size(summary.freed_bytes)
        );
    } else {
        println!(
            "Deleted {} folders ({} freed), {} errors",
            summary.deleted_count,
            format_size(summary.freed_bytes),
            summary.errors.len()
        );
        for error in &summary.errors {
            eprintln!(
                "  Error deleting {}: {}",
                error.path.display(),
                error.reason
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_delete_summary_new() {
        let summary = DeleteSummary::new();
        assert_eq!(summary.deleted_count, 0);
        assert_eq!(summary.freed_bytes, 0);
        assert!(summary.errors.is_empty());
    }

    #[test]
    fn test_delete_folders_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules/test.txt"), b"test").unwrap();

        let folders = vec![DiscoveredFolder {
            path: dir.join("node_modules"),
            ecosystem: "Node.js".to_string(),
            size_bytes: 100,
            selected: true,
        }];

        let summary = delete_folders(&folders, true);

        assert_eq!(summary.deleted_count, 1);
        assert_eq!(summary.freed_bytes, 100);
        assert!(summary.errors.is_empty());

        assert!(dir.join("node_modules").exists());
    }

    #[test]
    fn test_delete_folders_actual() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::create_dir_all(dir.join("target")).unwrap();
        fs::write(dir.join("target/test.txt"), b"test").unwrap();

        let folders = vec![DiscoveredFolder {
            path: dir.join("target"),
            ecosystem: "Rust".to_string(),
            size_bytes: 100,
            selected: true,
        }];

        let summary = delete_folders(&folders, false);

        assert_eq!(summary.deleted_count, 1);
        assert_eq!(summary.freed_bytes, 100);
        assert!(summary.errors.is_empty());

        assert!(!dir.join("target").exists());
    }

    #[test]
    fn test_delete_folders_skips_unselected() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::create_dir_all(dir.join("node_modules")).unwrap();

        let folders = vec![DiscoveredFolder {
            path: dir.join("node_modules"),
            ecosystem: "Node.js".to_string(),
            size_bytes: 100,
            selected: false,
        }];

        let summary = delete_folders(&folders, false);

        assert_eq!(summary.deleted_count, 0);
        assert!(summary.errors.is_empty());

        assert!(dir.join("node_modules").exists());
    }
}
