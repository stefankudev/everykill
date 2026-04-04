use crate::config::DiscoveredFolder;
use std::path::Path;

pub fn calculate_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0u64;

    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }

    Ok(total_size)
}

pub fn calculate_sizes(folders: &mut [DiscoveredFolder]) {
    use rayon::prelude::*;

    folders.par_iter_mut().for_each(|folder| {
        if let Ok(size) = calculate_size(&folder.path) {
            folder.size_bytes = size;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Ecosystem;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_size_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let size = calculate_size(temp_dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_size_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::write(dir.join("file1.txt"), b"hello").unwrap();
        fs::write(dir.join("file2.txt"), b"world!").unwrap();

        let size = calculate_size(dir).unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn test_calculate_sizes_parallel() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        let sub1 = dir.join("node_modules");
        let sub2 = dir.join("target");

        fs::create_dir_all(&sub1).unwrap();
        fs::create_dir_all(&sub2).unwrap();
        fs::write(sub1.join("a.txt"), b"12345").unwrap();
        fs::write(sub2.join("b.txt"), b"67890").unwrap();

        let ecosystems = vec![
            Ecosystem {
                name: "Node.js".to_string(),
                local: vec!["node_modules/".to_string()],
                global: vec![],
            },
            Ecosystem {
                name: "Rust".to_string(),
                local: vec!["target/".to_string()],
                global: vec![],
            },
        ];

        use crate::scanner::scan_directory;
        let mut folders = scan_directory(dir, &ecosystems);

        assert_eq!(folders.len(), 2);

        calculate_sizes(&mut folders);

        assert_eq!(folders[0].size_bytes, 5);
        assert_eq!(folders[1].size_bytes, 5);
    }
}
