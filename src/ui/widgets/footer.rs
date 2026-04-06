use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::size_util::format_size;
use crate::ui::app::{AppMode, AppState, ScanState};

pub struct FooterWidget<'a> {
    pub state: &'a AppState,
}

impl Widget for FooterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let lines = build_footer_lines(self.state);

        // Render up to area.height lines
        for (i, line) in lines.iter().enumerate() {
            if i as u16 >= area.height {
                break;
            }
            let row_area = Rect {
                x: area.x,
                y: area.y + i as u16,
                width: area.width,
                height: 1,
            };
            Paragraph::new(line.clone())
                .alignment(Alignment::Left)
                .render(row_area, buf);
        }
    }
}

fn build_footer_lines(state: &AppState) -> Vec<Line<'static>> {
    match &state.mode {
        AppMode::ConfirmDelete => confirm_lines(state),
        AppMode::FilterPopup => filter_hint_lines(state),
        AppMode::Normal => normal_lines(state),
    }
}

fn normal_lines(state: &AppState) -> Vec<Line<'static>> {
    // ---- line 1: stats ----
    let stats = match &state.scan_state {
        ScanState::Scanning => {
            let found = state.folders.len();
            format!(" Scanning…  ({} found so far)", found)
        }
        ScanState::Complete => {
            let total = state.folders.len();
            let sel = state.selected_count;
            let bytes = format_size(state.total_selected_bytes);
            let dry = if state.dry_run { "  [DRY-RUN]" } else { "" };
            format!(
                " {} folders found  │  Selected: {}  │  {} to free{}",
                total, sel, bytes, dry
            )
        }
        ScanState::Error(msg) => format!(" Error: {}", msg),
    };

    let stats_style = match &state.scan_state {
        ScanState::Error(_) => Style::default().fg(Color::Red),
        ScanState::Scanning => Style::default().fg(Color::Yellow),
        ScanState::Complete => Style::default(),
    };

    let line1 = Line::from(Span::styled(stats, stats_style));

    // ---- line 2: status message or keybinds ----
    let line2 = if let Some(msg) = &state.status_message {
        Line::from(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        keybind_line()
    };

    // ---- line 3: delete hint ----
    let line3 = if state.has_selected() {
        Line::from(Span::styled(
            " [Enter] delete selected",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(
            " [Enter] delete selected",
            Style::default().fg(Color::DarkGray),
        ))
    };

    vec![line1, line2, line3]
}

fn confirm_lines(state: &AppState) -> Vec<Line<'static>> {
    let bytes = format_size(state.total_selected_bytes);
    let dry = if state.dry_run { " [DRY-RUN]" } else { "" };
    let prompt = format!(
        " Delete {} folder(s) ({}){}? ",
        state.selected_count, bytes, dry
    );

    let line1 = Line::from(vec![
        Span::styled(
            prompt,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "[y]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" / "),
        Span::styled(
            "[N]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]);

    let line2 = Line::from(Span::styled(
        " [Esc] cancel",
        Style::default().fg(Color::DarkGray),
    ));

    vec![line1, line2, Line::from("")]
}

fn filter_hint_lines(_state: &AppState) -> Vec<Line<'static>> {
    let line1 = Line::from(Span::styled(
        " Filter popup open",
        Style::default().fg(Color::Cyan),
    ));
    let line2 = Line::from(Span::styled(
        " [↑↓/jk] navigate  [Space] toggle  [a] all  [n] none  [Esc/f] close",
        Style::default().fg(Color::DarkGray),
    ));
    vec![line1, line2, Line::from("")]
}

fn keybind_line() -> Line<'static> {
    Line::from(Span::styled(
        " [↑↓/jk] nav  [Space] select  [a] all  [n] none  [d] dry-run  [f] filter  [q] quit",
        Style::default().fg(Color::DarkGray),
    ))
}
