pub fn get_ascii_art(terminal_width: u16) -> &'static str {
    match terminal_width {
        0..=25 => include_str!("../../assets/ascii/w_025_h_003.txt"),
        26..=50 => include_str!("../../assets/ascii/w_050_h_004.txt"),
        51..=75 => include_str!("../../assets/ascii/w_075_h_005.txt"),
        76..=100 => include_str!("../../assets/ascii/w_100_h_006.txt"),
        101..=125 => include_str!("../../assets/ascii/w_125_h_006.txt"),
        126..=150 => include_str!("../../assets/ascii/w_150_h_007.txt"),
        151..=175 => include_str!("../../assets/ascii/w_175_h_007.txt"),
        176..=200 => include_str!("../../assets/ascii/w_200_h_008.txt"),
        201..=225 => include_str!("../../assets/ascii/w_225_h_008.txt"),
        _ => include_str!("../../assets/ascii/w_250_h_008.txt"),
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
