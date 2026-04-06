use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::config::Ecosystem;

#[derive(Debug, Clone, Default, Parser)]
#[command(
    author,
    version,
    about = "Find and remove dependency folders across all ecosystems"
)]
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

    /// Delete selected folders
    #[arg(short = 'D', long = "delete")]
    pub delete: bool,

    /// Disable the interactive TUI; print results as plain text
    #[arg(long = "no-tui", default_value_t = false)]
    pub no_tui: bool,
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
        if let Some(d) = self.depth {
            Some(d)
        } else if self.no_recursive {
            Some(0)
        } else {
            None
        }
    }

    /// Get ecosystems to scan
    pub fn get_ecosystems(&self, all_ecosystems: &[Ecosystem]) -> Vec<Ecosystem> {
        if self.all {
            all_ecosystems.to_vec()
        } else if let Some(targets) = &self.target {
            let target_names: Vec<String> = targets
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect();
            all_ecosystems
                .iter()
                .filter(|e| target_names.contains(&e.name.to_lowercase()))
                .cloned()
                .collect()
        } else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_excluded_dirs_none() {
        let args = Args {
            ..Default::default()
        };
        assert!(args.get_excluded_dirs().is_empty());
    }

    #[test]
    fn test_delete_flag_defaults_to_false() {
        let args = Args {
            ..Default::default()
        };
        assert!(!args.delete);
    }

    #[test]
    fn test_get_excluded_dirs_single() {
        let args = Args {
            exclude: Some("target".to_string()),
            ..Default::default()
        };
        assert_eq!(args.get_excluded_dirs(), vec!["target"]);
    }

    #[test]
    fn test_get_excluded_dirs_multiple() {
        let args = Args {
            exclude: Some("target,vendor,.next".to_string()),
            ..Default::default()
        };
        assert_eq!(args.get_excluded_dirs(), vec!["target", "vendor", ".next"]);
    }

    #[test]
    fn test_no_recursive_sets_depth_zero() {
        let args = Args {
            no_recursive: true,
            ..Default::default()
        };
        assert_eq!(args.get_depth_limit(), Some(0));
    }

    #[test]
    fn test_depth_takes_prescedence_over_no_recursive() {
        let args = Args {
            depth: Some(3),
            no_recursive: true,
            ..Default::default()
        };
        assert_eq!(args.get_depth_limit(), Some(3));
    }
}
