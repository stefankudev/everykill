use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::ui::ascii::get_ascii_art;

/// Number of lines in the ASCII art files.
pub const ART_HEIGHT: u16 = 4;

/// Returns the header height for a given terminal width:
/// 0 if the terminal is too narrow to show art, ART_HEIGHT otherwise.
pub fn header_height(terminal_width: u16) -> u16 {
    if get_ascii_art(terminal_width).is_some() {
        ART_HEIGHT
    } else {
        0
    }
}

/// Widget that renders the ASCII art banner, centred horizontally.
pub struct HeaderWidget {
    pub terminal_width: u16,
}

impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let Some(art) = get_ascii_art(self.terminal_width) else {
            return;
        };

        let style = Style::default().fg(Color::Red);

        for (row_idx, line_text) in art.lines().enumerate() {
            let y = area.top() + row_idx as u16;
            if y >= area.bottom() {
                break;
            }

            // Centre the line within the terminal width
            let line_len = line_text.len() as u16;
            let padding = if self.terminal_width > line_len {
                (self.terminal_width - line_len) / 2
            } else {
                0
            };

            let line = Line::from(vec![
                Span::raw(" ".repeat(padding as usize)),
                Span::styled(line_text, style),
            ]);

            buf.set_line(area.left(), y, &line, area.width);
        }
    }
}
