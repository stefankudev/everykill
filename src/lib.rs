pub mod ui;

pub fn run() {
    let width = ui::get_terminal_width();
    ui::print_centered_art(width);
}
