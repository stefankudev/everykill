# TUI Implementation Plan

Interactive terminal UI for everykill using [ratatui](https://ratatui.rs).

---

## Overview

The TUI is the default mode when running `everykill` interactively. It replaces the plain-text output in `run()` with a full-screen, keyboard- and mouse-driven interface that lets users browse discovered dependency folders, select which ones to delete, and confirm deletion — all without leaving the terminal.

Plain-text output is preserved via `--no-tui` for scripting, piping, and CI usage.

---

## Modes

| Mode | Trigger |
|------|---------|
| TUI (default) | `everykill` (stdout is a TTY and `--no-tui` is not set) |
| Plain text | `--no-tui` flag |

The existing `run()` flow in `lib.rs` becomes the plain-text path. A new `ui::tui::run_tui(args)` function handles the interactive path.

---

## New Dependencies

Add to `Cargo.toml`:

```toml
ratatui = "0.29"
```

`crossterm` is already a dependency and serves as ratatui's backend. No async runtime is needed — background scanning uses `std::thread` + `std::sync::mpsc` to keep the dependency footprint minimal.

---

## File Structure

```
src/ui/
├── mod.rs              update: export new modules
├── ascii.rs            unchanged
├── app.rs              NEW: AppState, ScanEvent, state mutation, tests
├── tui.rs              NEW: terminal init/restore, main event loop
└── widgets/
    ├── mod.rs          NEW
    ├── header.rs       NEW: ASCII art banner widget
    ├── list.rs         NEW: scrollable folder list (StatefulWidget)
    ├── footer.rs       NEW: status bar + keybind hints
    └── filter.rs       NEW: ecosystem filter popup
```

`src/args.rs`: add `--no-tui` flag.  
`src/lib.rs`: branch on `args.no_tui` to call either the existing plain-text path or `ui::tui::run_tui(args)`.

---

## CLI Changes

### `src/args.rs`

Add one field to `Args`:

```rust
/// Disable the interactive TUI; print results as plain text
#[arg(long, default_value_t = false)]
pub no_tui: bool,
```

### `src/lib.rs`

```rust
pub fn run() {
    let args = Args::parse();
    if args.no_tui {
        run_plain(args);   // existing logic, extracted to function
    } else {
        ui::tui::run_tui(args).expect("TUI failed");
    }
}
```

---

## AppState (`src/ui/app.rs`)

### Enums

```rust
pub enum ScanState {
    Scanning,       // background thread is running
    Complete,       // all folders found and sized
    Error(String),  // scan failed
}

pub enum AppMode {
    Normal,
    FilterPopup,                    // ecosystem filter overlay open
    ConfirmDelete,                  // delete confirmation prompt active
}
```

### Struct

```rust
pub struct AppState {
    // Data
    pub folders: Vec<DiscoveredFolder>,  // all discovered folders (unsorted)
    pub all_ecosystems: Vec<String>,     // unique ecosystem names, for popup

    // Navigation
    pub cursor: usize,         // index into visible_folders()
    pub scroll_offset: usize,  // top row index of the viewport

    // State
    pub scan_state: ScanState,
    pub mode: AppMode,
    pub dry_run: bool,

    // Filter
    pub active_ecosystem_filters: HashSet<String>,  // empty = show all
    pub filter_cursor: usize,                        // cursor in filter popup

    // Computed (recalculated on mutation)
    pub total_selected_bytes: u64,
    pub selected_count: usize,
}
```

### Key Methods

```rust
impl AppState {
    pub fn new() -> Self { ... }

    /// Folders visible after applying ecosystem filter
    pub fn visible_folders(&self) -> Vec<&DiscoveredFolder> { ... }

    /// Toggle selected on the folder at cursor
    pub fn toggle_selection(&mut self) { ... }

    /// Select all visible folders
    pub fn select_all(&mut self) { ... }

    /// Deselect all folders
    pub fn deselect_all(&mut self) { ... }

    /// Move cursor up, adjusting scroll_offset
    pub fn cursor_up(&mut self) { ... }

    /// Move cursor down, adjusting scroll_offset
    pub fn cursor_down(&mut self, viewport_height: usize) { ... }

    /// Recalculate total_selected_bytes and selected_count
    pub fn recalculate_totals(&mut self) { ... }

    /// Apply a ScanEvent from the background thread
    pub fn handle_scan_event(&mut self, event: ScanEvent) { ... }

    /// Toggle an ecosystem in/out of active_ecosystem_filters
    pub fn toggle_ecosystem_filter(&mut self, name: &str) { ... }
}
```

Mutation methods call `recalculate_totals()` after modifying `folders`.

### ScanEvent Channel

The background scan thread communicates with the main loop via:

```rust
pub enum ScanEvent {
    FolderFound(DiscoveredFolder),           // emitted as each folder is found
    SizeUpdated { path: PathBuf, bytes: u64 }, // emitted after size is calculated
    Done,
    Error(String),
}
```

The background thread:
1. Calls `scanner::scan_directory(...)` and sends `FolderFound` for each result
2. Calls `scanner::calculate_sizes(...)` on the full set and sends `SizeUpdated` for each
3. Sends `Done`

The main loop calls `rx.try_recv()` on every tick and delegates to `AppState::handle_scan_event`.

---

## Terminal Management (`src/ui/tui.rs`)

### Initialization

```rust
pub fn run_tui(args: Args) -> anyhow::Result<()> {
    // 1. Set panic hook to restore terminal before printing panic message
    // 2. enable_raw_mode()
    // 3. execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
    // 4. Construct CrosstermBackend + Terminal
    // 5. Spawn background scan thread
    // 6. Enter event loop
    // 7. On exit: restore terminal (disable_raw_mode, LeaveAlternateScreen, DisableMouseCapture)
}
```

### Panic Hook

```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    restore_terminal();   // always run before printing panic
    original_hook(info);
}));
```

### Event Loop

```rust
loop {
    // 1. Drain all pending ScanEvents from channel (non-blocking try_recv loop)
    // 2. terminal.draw(|f| render(f, &state))
    // 3. Poll crossterm events with timeout (16ms → ~60 fps)
    // 4. Dispatch keyboard / mouse events to handle_event(&mut state, event)
    // 5. if state.should_quit { break }
}
```

---

## Layout

The terminal is split into three vertical regions using ratatui `Layout`:

```
┌────────────────────────────────────────────────────────────┐
│                                                            │  ← Header (4 lines if width ≥ 44, else 0)
│    _______ _    _ _______  ______ __   __ _     _ _____   │
│    |______  \  /  |______ |_____/   \_/   |____/    |      │
│    |______   \/   |______ |    \_    |    |    \_ __|__    │
│                                                            │
├────────────────────────────────────────────────────────────┤
│  [x] ~/projects/app/node_modules      Node.js    245.3 MB  │  ← List (fills remaining space)
│  [ ] ~/projects/cli/target            Rust        88.1 MB  │
│  ▶ [ ] ~/projects/api/vendor          Go          12.4 MB  │  ← cursor row (highlighted)
│  [ ] ~/projects/java/build            Java         4.7 MB  │
│        ⣿ Scanning… (12 found)                              │  ← shown while scanning
├────────────────────────────────────────────────────────────┤
│  3 folders found │ Selected: 1 │ 245.3 MB to free          │  ← Footer line 1
│  [↑↓] nav  [Space] select  [a] all  [n] none  [d] dry-run  │  ← Footer line 2
│  [Enter] delete  [f] filter  [q] quit                      │  ← Footer line 3
└────────────────────────────────────────────────────────────┘
```

### Header Widget (`widgets/header.rs`)

- Calls `get_ascii_art(terminal_width)` from the existing `ascii.rs`
- If `None` (terminal too narrow), renders nothing (header height = 0)
- Otherwise wraps the art in a `ratatui::widgets::Paragraph`, centered horizontally
- Height: number of lines in the selected art file (currently 4)

### List Widget (`widgets/list.rs`)

Implemented as a `ratatui::widgets::Table` with a `TableState` for cursor tracking.

**Columns:**

| # | Content | Width |
|---|---------|-------|
| 1 | `[x]` / `[ ]` | 4 (fixed) |
| 2 | Path (truncated with `…` if overlong) | fills remaining |
| 3 | Ecosystem name | 14 (fixed) |
| 4 | Size (`…` if still 0 during scan) | 9 (fixed) |

**Row styles:**

| State | Style |
|-------|-------|
| Cursor row | `bg(Color::DarkGray) + BOLD` |
| Selected (`[x]`) | checkbox cell in `fg(Color::Green)` |
| Scanning (size = 0) | size cell shows `…` in dim style |
| Normal | default terminal colors |

Scroll is managed by keeping `scroll_offset` so that the cursor row is always visible. Page Up / Page Down move by `viewport_height - 1`.

### Footer Widget (`widgets/footer.rs`)

**Normal mode:**

```
Line 1:  {N} folders found  │  Selected: {M}  │  {X} to free   [DRY-RUN] (if dry_run)
Line 2:  [↑↓/jk] nav  [Space] select  [a] all  [n] none  [d] dry-run  [f] filter  [q] quit
Line 3:  [Enter] delete selected
```

**Scanning mode (ScanState::Scanning):**

```
Line 1:  Scanning…  ({N} found so far)
```

**Confirm delete mode:**

```
Line 1:  Delete {M} folders ({X})? [y / N]
Line 2:  (keybinds hidden)
```

**Error mode:**

```
Line 1:  Error: {message}  [q] quit
```

### Filter Popup Widget (`widgets/filter.rs`)

A floating centered box rendered on top of the list:

```
┌─── Filter by Ecosystem ────────────┐
│                                    │
│  [x] Node.js                       │
│  [x] Rust                          │
│  ▶ [ ] Go                          │  ← filter_cursor
│  [x] Python                        │
│  [ ] Java                          │
│                                    │
│  [Space] toggle  [a] all  [n] none │
│  [Esc] close                       │
└────────────────────────────────────┘
```

- Size: ~60% terminal width, ~70% terminal height, centered via `ratatui::layout::Rect`
- `active_ecosystem_filters` is a `HashSet<String>`; empty = all visible, non-empty = show only matching
- Changes apply immediately (list behind popup updates in real time)

---

## Key Bindings

### Normal Mode

| Key | Action |
|-----|--------|
| `↑` / `k` | Cursor up |
| `↓` / `j` | Cursor down |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Home` | Jump to first row |
| `End` | Jump to last row |
| `Space` | Toggle selection on cursor row |
| `a` | Select all visible folders |
| `n` | Deselect all folders |
| `d` | Toggle dry-run mode |
| `f` | Open ecosystem filter popup |
| `Enter` | Enter ConfirmDelete mode (if any selected) |
| `q` / `Q` / `Esc` | Quit |

### Confirm Delete Mode

| Key | Action |
|-----|--------|
| `y` / `Y` | Execute deletion (or print dry-run output), return to Normal |
| `n` / `N` / `Esc` | Cancel, return to Normal |

### Filter Popup Mode

| Key | Action |
|-----|--------|
| `↑` / `k` | Move popup cursor up |
| `↓` / `j` | Move popup cursor down |
| `Space` | Toggle ecosystem at popup cursor |
| `a` | Enable all ecosystems (clear filter) |
| `n` | Disable all (hide everything) |
| `Esc` / `f` | Close popup |

---

## Mouse Support

Mouse capture is enabled via `crossterm::event::EnableMouseCapture`.

| Event | Action |
|-------|--------|
| Left click on a list row | Move cursor to that row |
| Left click on `[ ]` / `[x]` cell | Move cursor + toggle selection |
| Scroll wheel up | Cursor up |
| Scroll wheel down | Cursor down |
| Left click on filter popup row | Move popup cursor to that row |
| Left click on filter popup checkbox | Toggle that ecosystem |

Mouse events use the row's screen `y` coordinate minus the list widget's top offset to determine which `DiscoveredFolder` was clicked.

---

## Deletion Flow

1. User presses `Enter` with at least one folder selected
2. `AppMode` transitions to `ConfirmDelete`
3. Footer shows: `Delete {M} folders ({X})? [y / N]`
4. If `y`: call `deleter::delete_folders(&folders, self.dry_run)`
   - Successful deletions: remove from `self.folders`, recalculate totals
   - Errors: show briefly in footer as `Error deleting {path}: {reason}`
5. If `n` or `Esc`: return to `AppMode::Normal` with no changes
6. After deletion completes, `ScanState` stays `Complete`; the list now shows only surviving folders

---

## Testing Strategy

Unit tests live in `src/ui/app.rs` (inline `#[cfg(test)]` modules). No full rendering tests are needed for the initial implementation.

| Test | What it covers |
|------|----------------|
| `test_toggle_selection` | Toggle selected on/off, totals update |
| `test_select_all` | All visible folders marked selected |
| `test_deselect_all` | All folders marked unselected |
| `test_visible_folders_no_filter` | Empty filter set → all folders visible |
| `test_visible_folders_with_filter` | Active filter → only matching ecosystem shown |
| `test_cursor_bounds` | cursor_up at 0 stays 0; cursor_down at end stays at end |
| `test_handle_scan_event_folder_found` | FolderFound appends to folders list |
| `test_handle_scan_event_size_updated` | SizeUpdated patches correct folder's size_bytes |
| `test_handle_scan_event_done` | Sets ScanState::Complete |
| `test_recalculate_totals` | Correct byte sum and count from mixed selected/unselected |

---

## Implementation Order

1. **`Cargo.toml`** — add `ratatui = "0.29"`
2. **`src/args.rs`** — add `--no-tui` flag
3. **`src/lib.rs`** — extract `run_plain()`, add TUI branch
4. **`src/ui/app.rs`** — `AppState`, `ScanEvent`, all state methods, unit tests
5. **`src/ui/widgets/mod.rs`** + **`header.rs`** — static banner render
6. **`src/ui/widgets/list.rs`** — scrollable table widget (static data first)
7. **`src/ui/widgets/footer.rs`** — status bar widget
8. **`src/ui/tui.rs`** — terminal init/restore, static render loop (no scan yet)
9. Wire background scan thread into `run_tui` and event loop
10. **`src/ui/widgets/filter.rs`** — ecosystem filter popup
11. Wire deletion confirmation flow
12. Add mouse event handling
13. Manual test pass across narrow, medium, and wide terminals
14. Update `docs/PROJECT_PLAN.md` to mark TUI complete

---

## Open Questions / Future Work

- **Windows portability**: `get_inode()` in `scanner/dir.rs` is Unix-only. Before a Windows release, replace with a platform-agnostic deduplication strategy.
- **`--no-tui` auto-detect**: A follow-up could auto-detect non-TTY stdout (piped output) and fall back to plain text automatically, making `--no-tui` only needed for explicit opt-out.
- **`--show-errors` flag**: Currently parsed but not wired. Plain-text path should respect it; TUI can surface errors inline.
- **`tempfile` in dev dependencies**: Move `tempfile` from `[dependencies]` to `[dev-dependencies]` to avoid including it in release builds.
- **Search/filter by path**: In addition to ecosystem filtering, a free-text path filter (type `/` to enter search mode) could be added in a follow-up.
