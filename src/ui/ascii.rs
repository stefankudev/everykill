pub fn get_ascii_art(terminal_width: u16) -> &'static str {
    match terminal_width {
        0..=25 => include_str!("../../ascii/width_025.txt"),
        26..=50 => include_str!("../../ascii/width_050.txt"),
        51..=75 => include_str!("../../ascii/width_075.txt"),
        76..=100 => include_str!("../../ascii/width_100.txt"),
        101..=125 => include_str!("../../ascii/width_125.txt"),
        126..=150 => include_str!("../../ascii/width_150.txt"),
        151..=175 => include_str!("../../ascii/width_175.txt"),
        176..=200 => include_str!("../../ascii/width_200.txt"),
        201..=225 => include_str!("../../ascii/width_225.txt"),
        _ => include_str!("../../ascii/width_250.txt"),
    }
}

pub fn print_centered_art(terminal_width: u16) {
    let art = get_ascii_art(terminal_width);
    for line in art.lines() {
        let padding = (terminal_width as usize).saturating_sub(line.len()) / 2;
        println!("{}{}", " ".repeat(padding), line);
    }
}

pub fn get_terminal_width() -> u16 {
    crossterm::terminal::size()
        .map(|(w, _)| w)
        .unwrap_or(80)
}
