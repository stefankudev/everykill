use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::ui::app::AppState;

pub struct FilterPopupWidget<'a> {
    pub state: &'a AppState,
}

impl Widget for FilterPopupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate popup dimensions: ~60% wide, ~70% tall, centred
        let popup_width = (area.width * 60 / 100).max(40).min(area.width);
        let popup_height = (area.height * 70 / 100).max(8).min(area.height);
        let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the background behind the popup
        Widget::render(Clear, popup_area, buf);

        // Draw border block
        let block = Block::default()
            .title(" Filter by Ecosystem ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup_area);
        Widget::render(block, popup_area, buf);

        // Render ecosystem list inside the block
        let list_height = inner.height.saturating_sub(2) as usize; // leave 2 rows for hints
        let ecosystems = &self.state.all_ecosystems;

        // Scroll so that filter_cursor is always visible
        let scroll_offset = if self.state.filter_cursor >= list_height {
            self.state.filter_cursor + 1 - list_height
        } else {
            0
        };

        let mut row = inner.y;

        for (i, name) in ecosystems.iter().enumerate() {
            if i < scroll_offset {
                continue;
            }
            if row >= inner.y + inner.height.saturating_sub(2) {
                break;
            }

            let is_cursor = i == self.state.filter_cursor;
            // Empty filter set means "show all" (all ecosystems are visible)
            let is_enabled = self.state.active_ecosystem_filters.is_empty()
                || !self.state.active_ecosystem_filters.contains(name.as_str());

            let checkbox = if is_enabled { "[x]" } else { "[ ]" };

            let checkbox_style = if is_enabled {
                Style::default().fg(Color::Green)
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

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(checkbox, checkbox_style),
                Span::raw(" "),
                Span::styled(name.clone(), row_style),
            ]);

            let cell_area = Rect {
                x: inner.x,
                y: row,
                width: inner.width,
                height: 1,
            };
            Paragraph::new(line).render(cell_area, buf);
            row += 1;
        }

        // Hint row at the bottom of the popup
        let hint_y = inner.y + inner.height.saturating_sub(1);
        let hint_area = Rect {
            x: inner.x,
            y: hint_y,
            width: inner.width,
            height: 1,
        };
        let hint = Line::from(Span::styled(
            " [Space] toggle  [a] all  [n] none  [Esc/f] close",
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(hint).render(hint_area, buf);
    }
}
