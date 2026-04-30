pub mod args;
pub mod config;
pub mod deleter;
pub mod scanner;
pub mod size_util;
pub mod ui;

use anyhow::Context;
use args::Args;
use clap::Parser;
use deleter::{delete_folders, print_delete_summary};
use size_util::format_size;

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.no_tui {
        run_plain(args)
    } else {
        ui::tui::run_tui(args)
    }
}

pub fn run_plain(args: Args) -> anyhow::Result<()> {
    let ecosystems =
        config::load_ecosystems().context("Failed to load ecosystem configurations")?;
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
            folders.sort_by_key(|b| std::cmp::Reverse(b.size_bytes));
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

    if args.delete {
        let folders: Vec<_> = folders
            .into_iter()
            .map(|mut f| {
                f.selected = true;
                f
            })
            .collect();
        let summary = delete_folders(&folders, false);
        print_delete_summary(&summary);
    } else {
        for folder in &folders {
            let uncertain_marker = if folder.confidence.is_uncertain() {
                "[?] "
            } else {
                ""
            };
            println!(
                "  {}{} ({}) - {}",
                uncertain_marker,
                folder.path.display(),
                folder.ecosystem,
                format_size(folder.size_bytes)
            );
        }
    }

    Ok(())
}
