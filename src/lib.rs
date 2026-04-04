pub mod config;
pub mod scanner;
pub mod size_util;
pub mod ui;

use size_util::format_size;
use std::path::Path;

pub fn run() {
    let width = ui::get_terminal_width();
    ui::print_centered_art(width);

    let ecosystems = config::load_ecosystems().expect("failed to load ecosystems");
    println!("Loaded {} ecosystems", ecosystems.len());

    let folders = scanner::scan_directory(Path::new("."), &ecosystems);
    println!("Found {} dependency folders", folders.len());

    let mut folders = folders;
    scanner::calculate_sizes(&mut folders);

    for folder in &folders {
        println!(
            "  {} ({}) - {}",
            folder.path.display(),
            folder.ecosystem,
            format_size(folder.size_bytes)
        );
    }
}
