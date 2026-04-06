use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Row, StatefulWidget, Table, TableState},
};

use crate::config::DiscoveredFolder;
use crate::size_util::format_size;
use crate::ui::app::{AppMode, AppState, ScanState};

// Column widths (fixed except path which fills the remainder)
const COL_CHECKBOX: u16 = 4; // "[x] "
const COL_ECOSYSTEM: u16 = 16;
const COL_SIZE: u16 = 10;

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        // keep the tail (more useful than the head for deep paths)
        let keep = max_len.saturating_sub(1);
        format!("…{}", &path[path.len() - keep..])
    }
}

fn format_size_cell(folder: &DiscoveredFolder, scan_complete: bool) -> String {
    if folder.size_bytes == 0 && !scan_complete {
        "…".to_string()
    } else {
        format_size(folder.size_bytes)
    }
}

/// Builds a `TableState` positioned at the current cursor / scroll_offset.
pub fn build_table_state(state: &AppState) -> TableState {
    let mut ts = TableState::default();
    if state.visible_len() > 0 {
        ts.select(Some(state.cursor));
        *ts.offset_mut() = state.scroll_offset;
    }
    ts
}

/// Stateful widget that renders the folder list as a scrollable table.
pub struct FolderListWidget<'a> {
    pub state: &'a AppState,
}

impl StatefulWidget for FolderListWidget<'_> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, table_state: &mut TableState) {
        let scan_complete = self.state.scan_state == ScanState::Complete;
        let visible = self.state.visible_folders();

        // Calculate path column width from whatever space is left
        let fixed = COL_CHECKBOX + COL_ECOSYSTEM + COL_SIZE + 3; // 3 separators
        let path_width = area.width.saturating_sub(fixed) as usize;

        let rows: Vec<Row> = visible
            .iter()
            .enumerate()
            .map(|(i, folder)| {
                let is_cursor = i == self.state.cursor;
                let is_selected = folder.selected;

                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let path_str = folder.path.display().to_string();
                let path_cell = truncate_path(&path_str, path_width);
                let eco_cell = folder.ecosystem.clone();
                let size_cell = format_size_cell(folder, scan_complete);

                // Style ecosystem cell: dim if uncertain (ambiguous/undetected)
                let eco_style = if folder.confidence.is_uncertain() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let checkbox_style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let row_style = if is_cursor {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let size_style = if folder.size_bytes == 0 && !scan_complete {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    ratatui::text::Text::from(Line::from(Span::styled(checkbox, checkbox_style))),
                    ratatui::text::Text::from(Line::from(Span::raw(path_cell))),
                    ratatui::text::Text::from(Line::from(Span::styled(eco_cell, eco_style))),
                    ratatui::text::Text::from(Line::from(Span::styled(size_cell, size_style))),
                ])
                .style(row_style)
            })
            .collect();

        // Show a "scanning" hint row while still scanning
        let mut all_rows = rows;
        if self.state.scan_state == ScanState::Scanning {
            let scanning_row = Row::new(vec![
                ratatui::text::Text::from(""),
                ratatui::text::Text::from(Line::from(Span::styled(
                    "  Scanning…",
                    Style::default().fg(Color::Yellow),
                ))),
                ratatui::text::Text::from(""),
                ratatui::text::Text::from(""),
            ]);
            all_rows.push(scanning_row);
        }

        let table = Table::new(
            all_rows,
            [
                Constraint::Length(COL_CHECKBOX),
                Constraint::Fill(1),
                Constraint::Length(COL_ECOSYSTEM),
                Constraint::Length(COL_SIZE),
            ],
        )
        .column_spacing(1)
        // Highlight the selected row (cursor) with an override style
        .row_highlight_style(if self.state.mode == AppMode::Normal {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        });

        StatefulWidget::render(table, area, buf, table_state);
    }
}
