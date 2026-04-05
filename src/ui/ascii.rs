pub fn get_ascii_art(terminal_width: u16) -> Option<&'static str> {
    if terminal_width < 44 {
        None
    } else if terminal_width <= 68 {
        Some(include_str!("../../assets/ascii/44.txt"))
    } else {
        Some(include_str!("../../assets/ascii/68.txt"))
    }
}

pub fn print_centered_art(terminal_width: u16) {
    if let Some(art) = get_ascii_art(terminal_width) {
        for line in art.lines() {
            let padding = (terminal_width as usize).saturating_sub(line.len()) / 2;
            println!("{}{}", " ".repeat(padding), line);
        }
    }
}

pub fn get_terminal_width() -> u16 {
    crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80)
}
