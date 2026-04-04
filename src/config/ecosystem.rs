use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ecosystem {
    pub name: String,
    pub local: Vec<String>,
    pub global: Vec<String>,
}

impl Ecosystem {
    pub fn matches_folder(&self, folder_name: &str) -> bool {
        self.local
            .iter()
            .any(|p| match_folder_pattern(p, folder_name))
            || self
                .global
                .iter()
                .any(|p| match_folder_pattern(p, folder_name))
    }
}

fn match_folder_pattern(pattern: &str, folder_name: &str) -> bool {
    let pattern = pattern.trim_end_matches('/');
    folder_name == pattern
}

#[derive(Debug, Clone)]
pub struct DiscoveredFolder {
    pub path: PathBuf,
    pub ecosystem: String,
    pub size_bytes: u64,
    pub selected: bool,
}

impl DiscoveredFolder {
    pub fn new(path: PathBuf, ecosystem: String) -> Self {
        Self {
            path,
            ecosystem,
            size_bytes: 0,
            selected: false,
        }
    }
}

pub fn load_ecosystems() -> anyhow::Result<Vec<Ecosystem>> {
    let mut ecosystems = Vec::new();
    for entry in glob::glob("ecosystems/*.json")? {
        match entry {
            Ok(path) => match load_ecosystem_from_path(&path) {
                Ok(eco) => ecosystems.push(eco),
                Err(e) => eprintln!("Warning: failed to load {:?}: {}", path, e),
            },
            Err(e) => eprintln!("Warning: glob error: {}", e),
        }
    }
    Ok(ecosystems)
}

pub fn load_ecosystem(name: &str) -> anyhow::Result<Ecosystem> {
    let path = Path::new("ecosystems").join(format!("{}.json", name));
    load_ecosystem_from_path(&path)
}

fn load_ecosystem_from_path(path: &Path) -> anyhow::Result<Ecosystem> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {:?}", path))?;
    let ecosystem: Ecosystem =
        serde_json::from_str(&content).with_context(|| format!("failed to parse {:?}", path))?;
    Ok(ecosystem)
}

pub fn build_ecosystem_cache(ecosystems: &[Ecosystem]) -> HashMap<String, Ecosystem> {
    ecosystems
        .iter()
        .map(|e| (e.name.clone(), e.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_folder_exact() {
        let eco = Ecosystem {
            name: "Node.js".to_string(),
            local: vec!["node_modules/".to_string()],
            global: vec!["~/.npm/".to_string()],
        };
        assert!(eco.matches_folder("node_modules"));
        assert!(!eco.matches_folder("node_modules123"));
    }

    #[test]
    fn test_matches_folder_multiple_patterns() {
        let eco = Ecosystem {
            name: "Rust".to_string(),
            local: vec!["target/".to_string(), "Cargo.lock".to_string()],
            global: vec![],
        };
        assert!(eco.matches_folder("target"));
        assert!(eco.matches_folder("Cargo.lock"));
        assert!(!eco.matches_folder("cargo"));
    }

    #[test]
    fn test_discovered_folder_new() {
        let path = PathBuf::from("/some/path/node_modules");
        let folder = DiscoveredFolder::new(path.clone(), "Node.js".to_string());
        assert_eq!(folder.path, path);
        assert_eq!(folder.ecosystem, "Node.js");
        assert_eq!(folder.size_bytes, 0);
        assert!(!folder.selected);
    }
}
