use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How certain we are that a discovered folder belongs to an ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Confidence {
    #[default]
    Certain,
    Confirmed,
    Ambiguous,
    Undetected,
}

impl Confidence {
    pub fn is_uncertain(self) -> bool {
        matches!(self, Confidence::Ambiguous | Confidence::Undetected)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ecosystem {
    pub name: String,
    pub local: Vec<String>,
    pub global: Vec<String>,
    /// Files/dirs in the project root that confirm this ecosystem (e.g. "Cargo.toml").
    /// Empty means the ecosystem has no ambiguous patterns and needs no confirmation.
    #[serde(default)]
    pub markers: Vec<String>,
}

impl Ecosystem {
    pub fn matches_folder(&self, folder_name: &str) -> bool {
        self.matches_folder_with_globals(folder_name, true)
    }

    pub fn matches_folder_with_globals(&self, folder_name: &str, include_globals: bool) -> bool {
        let local_match = self
            .local
            .iter()
            .any(|p| match_folder_pattern(p, folder_name));

        if local_match {
            return true;
        }

        if include_globals {
            return self
                .global
                .iter()
                .any(|p| match_folder_pattern(p, folder_name));
        }

        false
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
    pub confidence: Confidence,
}

impl DiscoveredFolder {
    pub fn new(path: PathBuf, ecosystem: String) -> Self {
        Self {
            path,
            ecosystem,
            size_bytes: 0,
            selected: false,
            confidence: Confidence::Certain,
        }
    }

    pub fn with_resolution(path: PathBuf, ecosystem: String, confidence: Confidence) -> Self {
        Self {
            path,
            ecosystem,
            size_bytes: 0,
            selected: false,
            confidence,
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
            markers: vec![],
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
            markers: vec![],
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
