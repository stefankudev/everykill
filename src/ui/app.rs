use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::DiscoveredFolder;

// ---------------------------------------------------------------------------
// Scan state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ScanState {
    Scanning,
    Complete,
    Error(String),
}

// ---------------------------------------------------------------------------
// App mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    FilterPopup,
    ConfirmDelete,
}

// ---------------------------------------------------------------------------
// Events sent from background scan thread
// ---------------------------------------------------------------------------

pub enum ScanEvent {
    FolderFound(DiscoveredFolder),
    SizeUpdated { path: PathBuf, bytes: u64 },
    Done,
    Error(String),
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

pub struct AppState {
    /// All discovered folders (unfiltered)
    pub folders: Vec<DiscoveredFolder>,
    /// Unique ecosystem names across all discovered folders (for filter popup)
    pub all_ecosystems: Vec<String>,

    /// Cursor index into `visible_indices()`
    pub cursor: usize,
    /// First visible row index (into `visible_indices()`) for scrolling
    pub scroll_offset: usize,

    pub scan_state: ScanState,
    pub mode: AppMode,
    pub dry_run: bool,

    /// If non-empty, only show folders whose ecosystem is in this set
    pub active_ecosystem_filters: HashSet<String>,
    /// Cursor position inside the filter popup list
    pub filter_cursor: usize,

    /// Computed totals (recalculated after every mutation)
    pub total_selected_bytes: u64,
    pub selected_count: usize,

    /// Transient status message shown in footer (errors, dry-run output, etc.)
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            all_ecosystems: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            scan_state: ScanState::Scanning,
            mode: AppMode::Normal,
            dry_run: false,
            active_ecosystem_filters: HashSet::new(),
            filter_cursor: 0,
            total_selected_bytes: 0,
            selected_count: 0,
            status_message: None,
        }
    }

    // -----------------------------------------------------------------------
    // Visibility helpers
    // -----------------------------------------------------------------------

    /// Indices into `self.folders` that pass the current ecosystem filter.
    pub fn visible_indices(&self) -> Vec<usize> {
        if self.active_ecosystem_filters.is_empty() {
            (0..self.folders.len()).collect()
        } else {
            self.folders
                .iter()
                .enumerate()
                .filter(|(_, f)| self.active_ecosystem_filters.contains(&f.ecosystem))
                .map(|(i, _)| i)
                .collect()
        }
    }

    /// Visible folders as immutable references.
    pub fn visible_folders(&self) -> Vec<&DiscoveredFolder> {
        self.visible_indices()
            .iter()
            .map(|&i| &self.folders[i])
            .collect()
    }

    pub fn visible_len(&self) -> usize {
        self.visible_indices().len()
    }

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    pub fn toggle_selection(&mut self) {
        let indices = self.visible_indices();
        if let Some(&real_idx) = indices.get(self.cursor) {
            self.folders[real_idx].selected = !self.folders[real_idx].selected;
            self.recalculate_totals();
        }
    }

    pub fn select_all(&mut self) {
        for &i in &self.visible_indices() {
            self.folders[i].selected = true;
        }
        self.recalculate_totals();
    }

    pub fn deselect_all(&mut self) {
        for f in &mut self.folders {
            f.selected = false;
        }
        self.recalculate_totals();
    }

    // -----------------------------------------------------------------------
    // Navigation
    // -----------------------------------------------------------------------

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.clamp_scroll(1); // viewport_height placeholder; tui.rs passes real value
    }

    pub fn cursor_down(&mut self, viewport_height: usize) {
        let len = self.visible_len();
        if len > 0 && self.cursor < len - 1 {
            self.cursor += 1;
        }
        self.clamp_scroll(viewport_height);
    }

    pub fn page_up(&mut self, viewport_height: usize) {
        self.cursor = self.cursor.saturating_sub(viewport_height);
        self.clamp_scroll(viewport_height);
    }

    pub fn page_down(&mut self, viewport_height: usize) {
        let len = self.visible_len();
        if len > 0 {
            self.cursor = (self.cursor + viewport_height).min(len - 1);
        }
        self.clamp_scroll(viewport_height);
    }

    pub fn jump_to_top(&mut self) {
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        let len = self.visible_len();
        if len > 0 {
            self.cursor = len - 1;
        }
        self.clamp_scroll(1);
    }

    /// Adjust scroll_offset so cursor stays within the viewport.
    pub fn clamp_scroll(&mut self, viewport_height: usize) {
        let vh = viewport_height.max(1);
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + vh {
            self.scroll_offset = self.cursor + 1 - vh;
        }
    }

    /// Move cursor to a specific visible-list row (e.g. from mouse click).
    pub fn set_cursor(&mut self, row: usize, viewport_height: usize) {
        let len = self.visible_len();
        if len > 0 {
            self.cursor = row.min(len - 1);
        }
        self.clamp_scroll(viewport_height);
    }

    // -----------------------------------------------------------------------
    // Totals
    // -----------------------------------------------------------------------

    pub fn recalculate_totals(&mut self) {
        self.total_selected_bytes = self
            .folders
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.size_bytes)
            .sum();
        self.selected_count = self.folders.iter().filter(|f| f.selected).count();
    }

    // -----------------------------------------------------------------------
    // Scan events from background thread
    // -----------------------------------------------------------------------

    pub fn handle_scan_event(&mut self, event: ScanEvent) {
        match event {
            ScanEvent::FolderFound(folder) => {
                // Track new ecosystem name
                if !self.all_ecosystems.contains(&folder.ecosystem) {
                    self.all_ecosystems.push(folder.ecosystem.clone());
                    self.all_ecosystems.sort();
                }
                self.folders.push(folder);
            }
            ScanEvent::SizeUpdated { path, bytes } => {
                if let Some(f) = self.folders.iter_mut().find(|f| f.path == path) {
                    f.size_bytes = bytes;
                }
                self.recalculate_totals();
            }
            ScanEvent::Done => {
                self.scan_state = ScanState::Complete;
                self.recalculate_totals();
            }
            ScanEvent::Error(msg) => {
                self.scan_state = ScanState::Error(msg);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Ecosystem filter (popup)
    // -----------------------------------------------------------------------

    pub fn toggle_ecosystem_filter(&mut self, name: &str) {
        if self.active_ecosystem_filters.contains(name) {
            self.active_ecosystem_filters.remove(name);
        } else {
            self.active_ecosystem_filters.insert(name.to_string());
        }
        // Clamp cursor after filter change
        let len = self.visible_len();
        if len == 0 {
            self.cursor = 0;
        } else if self.cursor >= len {
            self.cursor = len - 1;
        }
    }

    pub fn filter_select_all(&mut self) {
        self.active_ecosystem_filters.clear();
    }

    pub fn filter_deselect_all(&mut self) {
        for name in &self.all_ecosystems.clone() {
            self.active_ecosystem_filters.insert(name.clone());
        }
    }

    pub fn filter_cursor_up(&mut self) {
        if self.filter_cursor > 0 {
            self.filter_cursor -= 1;
        }
    }

    pub fn filter_cursor_down(&mut self) {
        if !self.all_ecosystems.is_empty() && self.filter_cursor < self.all_ecosystems.len() - 1 {
            self.filter_cursor += 1;
        }
    }

    pub fn toggle_filter_at_cursor(&mut self) {
        if let Some(name) = self.all_ecosystems.get(self.filter_cursor).cloned() {
            self.toggle_ecosystem_filter(&name);
        }
    }

    // -----------------------------------------------------------------------
    // Deletion helpers
    // -----------------------------------------------------------------------

    /// Returns true if any visible folder is selected.
    pub fn has_selected(&self) -> bool {
        self.selected_count > 0
    }

    /// Remove folders whose paths are in the given set (after successful deletion).
    pub fn remove_deleted(&mut self, deleted_paths: &HashSet<PathBuf>) {
        self.folders.retain(|f| !deleted_paths.contains(&f.path));
        // Clamp cursor
        let len = self.visible_len();
        if len == 0 {
            self.cursor = 0;
            self.scroll_offset = 0;
        } else if self.cursor >= len {
            self.cursor = len - 1;
        }
        self.recalculate_totals();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_folder(path: &str, ecosystem: &str, size: u64) -> DiscoveredFolder {
        let mut f = DiscoveredFolder::new(PathBuf::from(path), ecosystem.to_string());
        f.size_bytes = size;
        f
    }

    fn state_with_folders() -> AppState {
        let mut s = AppState::new();
        s.handle_scan_event(ScanEvent::FolderFound(make_folder(
            "/a/node_modules",
            "Node.js",
            1000,
        )));
        s.handle_scan_event(ScanEvent::FolderFound(make_folder(
            "/b/target",
            "Rust",
            2000,
        )));
        s.handle_scan_event(ScanEvent::FolderFound(make_folder("/c/vendor", "Go", 500)));
        s.recalculate_totals();
        s
    }

    #[test]
    fn test_toggle_selection() {
        let mut s = state_with_folders();
        assert!(!s.folders[0].selected);
        s.toggle_selection();
        assert!(s.folders[0].selected);
        assert_eq!(s.selected_count, 1);
        assert_eq!(s.total_selected_bytes, 1000);
        s.toggle_selection();
        assert!(!s.folders[0].selected);
        assert_eq!(s.selected_count, 0);
    }

    #[test]
    fn test_select_all() {
        let mut s = state_with_folders();
        s.select_all();
        assert!(s.folders.iter().all(|f| f.selected));
        assert_eq!(s.selected_count, 3);
        assert_eq!(s.total_selected_bytes, 3500);
    }

    #[test]
    fn test_deselect_all() {
        let mut s = state_with_folders();
        s.select_all();
        s.deselect_all();
        assert!(s.folders.iter().all(|f| !f.selected));
        assert_eq!(s.selected_count, 0);
        assert_eq!(s.total_selected_bytes, 0);
    }

    #[test]
    fn test_visible_folders_no_filter() {
        let s = state_with_folders();
        assert_eq!(s.visible_len(), 3);
    }

    #[test]
    fn test_visible_folders_with_filter() {
        let mut s = state_with_folders();
        s.active_ecosystem_filters.insert("Rust".to_string());
        assert_eq!(s.visible_len(), 1);
        assert_eq!(s.visible_folders()[0].ecosystem, "Rust");
    }

    #[test]
    fn test_cursor_bounds_up() {
        let mut s = state_with_folders();
        s.cursor = 0;
        s.cursor_up();
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_cursor_bounds_down() {
        let mut s = state_with_folders();
        s.cursor = 2;
        s.cursor_down(10);
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_handle_scan_event_folder_found() {
        let mut s = AppState::new();
        s.handle_scan_event(ScanEvent::FolderFound(make_folder(
            "/x/node_modules",
            "Node.js",
            0,
        )));
        assert_eq!(s.folders.len(), 1);
        assert!(s.all_ecosystems.contains(&"Node.js".to_string()));
    }

    #[test]
    fn test_handle_scan_event_size_updated() {
        let mut s = AppState::new();
        s.handle_scan_event(ScanEvent::FolderFound(make_folder(
            "/x/node_modules",
            "Node.js",
            0,
        )));
        s.handle_scan_event(ScanEvent::SizeUpdated {
            path: PathBuf::from("/x/node_modules"),
            bytes: 9999,
        });
        assert_eq!(s.folders[0].size_bytes, 9999);
    }

    #[test]
    fn test_handle_scan_event_done() {
        let mut s = AppState::new();
        s.handle_scan_event(ScanEvent::Done);
        assert_eq!(s.scan_state, ScanState::Complete);
    }

    #[test]
    fn test_recalculate_totals() {
        let mut s = state_with_folders();
        s.folders[0].selected = true;
        s.folders[2].selected = true;
        s.recalculate_totals();
        assert_eq!(s.selected_count, 2);
        assert_eq!(s.total_selected_bytes, 1500);
    }
}
