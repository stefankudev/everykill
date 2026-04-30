use std::collections::HashSet;
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

use crate::args::Args;
use crate::config;
use crate::deleter::delete_folders;
use crate::scanner;
use crate::ui::app::{AppMode, AppState, ScanEvent};
use crate::ui::widgets::{
    filter::FilterPopupWidget,
    footer::FooterWidget,
    header::header_height,
    header::HeaderWidget,
    list::{build_table_state, FolderListWidget},
};

// Tick rate for the event loop (~60 fps)
const TICK_MS: u64 = 16;

// Guard to prevent double-wrapping of panic hook
static PANIC_HOOK_SET: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn run_tui(args: Args) -> anyhow::Result<()> {
    // Install panic hook to restore terminal before printing panic details
    // Use atomic flag to prevent re-wrapping on multiple invocations
    if !PANIC_HOOK_SET.swap(true, Ordering::SeqCst) {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = restore_terminal_raw();
            original_hook(info);
        }));
    }

    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, args);

    // Always restore terminal, even on error
    restore_terminal(&mut terminal)?;
    result
}

// ---------------------------------------------------------------------------
// Terminal setup / restore
// ---------------------------------------------------------------------------

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Version that works without a Terminal handle (for panic hook).
fn restore_terminal_raw() -> io::Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Background scan thread
// ---------------------------------------------------------------------------

fn spawn_scan_thread(args: &Args) -> Receiver<ScanEvent> {
    let (tx, rx) = mpsc::channel::<ScanEvent>();
    let args_clone = args.clone();

    thread::spawn(move || {
        // Load ecosystems inside thread to avoid Send issues with the main load
        let ecosystems = match config::load_ecosystems() {
            Ok(e) => e,
            Err(err) => {
                let _ = tx.send(ScanEvent::Error(err.to_string()));
                eprintln!("Warning: scan thread failed to load ecosystems: {}", err);
                return;
            }
        };

        let target_ecosystems = args_clone.get_ecosystems(&ecosystems);

        // Scan
        let folders = scanner::scan_directory(
            &args_clone.get_scan_path(),
            &target_ecosystems,
            args_clone.should_include_globals(),
            &args_clone.get_excluded_dirs(),
            args_clone.exclude_hidden,
            args_clone.get_depth_limit(),
        );

        // Send each folder as it's "found" (scan_directory returns a batch, so we stream them)
        for folder in &folders {
            if tx.send(ScanEvent::FolderFound(folder.clone())).is_err() {
                eprintln!("Debug: scan thread exiting - receiver dropped (user quit)");
                return;
            }
        }

        // Calculate sizes — do it ourselves here and send updates
        let mut sized = folders;
        scanner::calculate_sizes(&mut sized);

        for folder in &sized {
            if tx
                .send(ScanEvent::SizeUpdated {
                    path: folder.path.clone(),
                    bytes: folder.size_bytes,
                })
                .is_err()
            {
                eprintln!("Debug: scan thread exiting - receiver dropped (user quit)");
                return;
            }
        }

        let _ = tx.send(ScanEvent::Done);
    });

    rx
}

// ---------------------------------------------------------------------------
// Main application loop
// ---------------------------------------------------------------------------

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, args: Args) -> anyhow::Result<()> {
    let mut state = AppState::new();
    let rx = spawn_scan_thread(&args);
    let mut should_quit = false;

    while !should_quit {
        let tick_start = Instant::now();

        // Drain all pending scan events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            state.handle_scan_event(event);
        }

        // Clear expired status messages
        state.clear_expired_status();

        // Render
        let term_size = terminal.size().unwrap_or_default();
        let term_width = term_size.width;
        terminal.draw(|frame| {
            let area = frame.area();
            let viewport_height = compute_list_height(area.height, term_width) as usize;
            render(frame, &mut state, term_width, viewport_height);
        })?;

        // Poll for input with remaining tick budget
        let elapsed = tick_start.elapsed();
        let timeout = Duration::from_millis(TICK_MS).saturating_sub(elapsed);

        if event::poll(timeout)? {
            let vp_height = compute_list_height(term_size.height, term_width) as usize;
            let header_h = header_height(term_width) as usize;

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    should_quit = handle_key(&mut state, key.code, vp_height);
                }
                Event::Mouse(mouse) => {
                    handle_mouse(&mut state, mouse, header_h, vp_height);
                }
                Event::Resize(_, _) => {
                    // Handled automatically by ratatui on next draw
                }
                _ => {}
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

fn compute_list_height(total_height: u16, term_width: u16) -> u16 {
    let hh = header_height(term_width);
    let footer_h: u16 = 3;
    total_height.saturating_sub(hh + footer_h)
}

fn build_layout(area: Rect, term_width: u16) -> (Rect, Rect, Rect) {
    let hh = header_height(term_width);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(hh),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(
    frame: &mut ratatui::Frame,
    state: &mut AppState,
    term_width: u16,
    _viewport_height: usize,
) {
    let area = frame.area();
    let (header_area, list_area, footer_area) = build_layout(area, term_width);

    // Header
    frame.render_widget(
        HeaderWidget {
            terminal_width: term_width,
        },
        header_area,
    );

    // List — we own scroll_offset; pass it to ratatui, never read it back
    let mut table_state = build_table_state(state);
    frame.render_stateful_widget(FolderListWidget { state }, list_area, &mut table_state);

    // Footer
    frame.render_widget(FooterWidget { state }, footer_area);

    // Filter popup (rendered on top)
    if state.mode == AppMode::FilterPopup {
        frame.render_widget(FilterPopupWidget { state }, area);
    }
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

fn handle_key(state: &mut AppState, key: KeyCode, viewport_height: usize) -> bool {
    match &state.mode {
        AppMode::Normal => handle_key_normal(state, key, viewport_height),
        AppMode::FilterPopup => {
            handle_key_filter(state, key);
            false
        }
        AppMode::ConfirmDelete => handle_key_confirm(state, key),
    }
}

fn handle_key_normal(state: &mut AppState, key: KeyCode, viewport_height: usize) -> bool {
    match key {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => state.cursor_up(viewport_height),
        KeyCode::Down | KeyCode::Char('j') => state.cursor_down(viewport_height),
        KeyCode::PageUp => state.page_up(viewport_height),
        KeyCode::PageDown => state.page_down(viewport_height),
        KeyCode::Home => state.jump_to_top(),
        KeyCode::End => state.jump_to_bottom(viewport_height),

        // Selection
        KeyCode::Char(' ') => state.toggle_selection(),
        KeyCode::Char('a') => state.select_all(),
        KeyCode::Char('n') => state.deselect_all(),

        // Toggles
        KeyCode::Char('d') => {
            state.dry_run = !state.dry_run;
            let msg = if state.dry_run {
                "Dry-run ON — deletions will be simulated".to_string()
            } else {
                "Dry-run OFF — deletions are permanent".to_string()
            };
            state.show_status(msg);
        }

        // Filter popup
        KeyCode::Char('f') => {
            state.mode = AppMode::FilterPopup;
        }

        // Delete
        KeyCode::Enter => {
            if state.has_selected() {
                state.mode = AppMode::ConfirmDelete;
            } else {
                state.status_message =
                    Some("No folders selected — use Space to select".to_string());
            }
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return true,

        _ => {}
    }
    false
}

fn handle_key_filter(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => state.filter_cursor_up(),
        KeyCode::Down | KeyCode::Char('j') => state.filter_cursor_down(),
        KeyCode::Char(' ') => state.toggle_filter_at_cursor(),
        KeyCode::Char('a') => state.filter_select_all(),
        KeyCode::Char('n') => state.filter_deselect_all(),
        KeyCode::Esc | KeyCode::Char('f') => state.mode = AppMode::Normal,
        _ => {}
    }
}

fn handle_key_confirm(state: &mut AppState, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            execute_deletion(state);
            state.mode = AppMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}

// ---------------------------------------------------------------------------
// Deletion execution
// ---------------------------------------------------------------------------

fn execute_deletion(state: &mut AppState) {
    let summary = delete_folders(&state.folders, state.dry_run);

    if state.dry_run {
        state.show_status(format!(
            "Dry-run: would free {} from {} folder(s)",
            crate::size_util::format_size(summary.freed_bytes),
            summary.deleted_count
        ));
    } else {
        // Remove successfully deleted folders from the list
        let deleted_paths: HashSet<_> = state
            .folders
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.path.clone())
            .collect();
        state.remove_deleted(&deleted_paths);

        let msg = if summary.errors.is_empty() {
            format!(
                "Deleted {} folder(s), freed {}",
                summary.deleted_count,
                crate::size_util::format_size(summary.freed_bytes)
            )
        } else {
            format!(
                "Deleted {} folder(s), {} error(s) — check stderr",
                summary.deleted_count,
                summary.errors.len()
            )
        };
        state.show_status(msg);
    }
}

// ---------------------------------------------------------------------------
// Mouse handling
// ---------------------------------------------------------------------------

fn handle_mouse(
    state: &mut AppState,
    mouse: crossterm::event::MouseEvent,
    header_height: usize,
    viewport_height: usize,
) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            state.cursor_up(viewport_height);
        }
        MouseEventKind::ScrollDown => {
            state.cursor_down(viewport_height);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            let row = mouse.row as usize;
            // Offset by header height (header rows are not clickable list rows)
            if row < header_height {
                return;
            }
            let list_row = row - header_height;
            let visible_row = list_row + state.scroll_offset;

            if state.mode == AppMode::FilterPopup {
                // Clicks inside the popup move the filter cursor
                // (Simple heuristic: treat click row as filter index)
                if visible_row < state.all_ecosystems.len() {
                    state.filter_cursor = visible_row;
                    state.toggle_filter_at_cursor();
                }
                return;
            }

            if visible_row < state.visible_len() {
                state.set_cursor(visible_row, viewport_height);
                // Click on checkbox column (columns 0-3) toggles selection
                if mouse.column < 4 {
                    state.toggle_selection();
                }
            }
        }
        _ => {}
    }
}
