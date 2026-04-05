pub mod args;
pub mod config;
pub mod scanner;
pub mod size_util;
pub mod ui;

use args::Args;
use clap::Parser;
use size_util::format_size;

pub fn run() {
    let args = Args::parse();

    let ecosystems = config::load_ecosystems().expect("failed to load ecosystems");
    let target_ecosystems = args.get_ecosystems(&ecosystems);
    let include_globals = args.should_include_globals();
    let excluded_dirs = args.get_excluded_dirs();

    let folders = scanner::scan_directory(
        &args.get_scan_path(),
        &target_ecosystems,
        include_globals,
        &excluded_dirs,
        args.exclude_hidden,
        args.get_depth_limit(),
    );

    let mut folders = folders;
    scanner::calculate_sizes(&mut folders);

    let folders = match args.sort {
        Some(args::SortBy::Size) => {
            let mut folders = folders;
            folders.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            folders
        }
        Some(args::SortBy::Path) => {
            let mut folders = folders;
            folders.sort_by(|a, b| a.path.cmp(&b.path));
            folders
        }
        None => folders,
    };

    println!("Found {} dependency folders", folders.len());

    for folder in &folders {
        println!(
            "  {} ({}) - {}",
            folder.path.display(),
            folder.ecosystem,
            format_size(folder.size_bytes)
        );
    }
}
