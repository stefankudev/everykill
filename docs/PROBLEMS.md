# Everykill - Issues & Code Review Combined

**Last Updated:** 2026-04-12

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [High-Priority Issues (Fix Before v1.0)](#high-priority-issues-fix-before-v10)
3. [Medium-Priority Issues (Follow-Up PRs)](#medium-priority-issues-follow-up-prs)
4. [Low-Priority Issues (Optional)](#low-priority-issues-optional)
5. [Epic Issues / Feature Gaps](#epic-issues--feature-gaps)

---

## Executive Summary

The everykill project demonstrates **excellent software engineering practices** with a well-architected Rust codebase, comprehensive testing (47 unit tests), and smart algorithms. All **critical issues** have been resolved for the v1.0 release.

Additionally, **3 high-priority issues** affect error handling and UX, and **11 medium-priority issues** need attention for robustness.

Beyond code quality issues, there are **6 major feature gaps** from PROBLEMS.md that need to be addressed for a complete v1.0 release (CI/CD pipelines, binary building, ecosystem markers, marketing).

**With ~2 hours of focused work** on critical/high items + **addressing the epic features**, this project will be ready for v1.0.

---

## High-Priority Issues (Fix Before v1.0)

### 🟠 HIGH #5: Status Messages Cleared Immediately

**Status:** ❌ UNSOLVED  
**Location:** `src/ui/tui.rs:281-283`  
**Severity:** HIGH  
**Effort:** 30 minutes

**Problem:**

Every keystroke immediately clears the status message:
```rust
fn handle_key(state: &mut AppState, key: KeyCode, viewport_height: usize) -> bool {
    state.status_message = None;  // ← CLEARED ON EVERY KEY!
    match &state.mode { ... }
}
```

**What happens:**
1. User deletes folders → status shows "Deleted 5 folders, freed 250MB"
2. User presses any key (even arrow) → status is IMMEDIATELY cleared
3. User can't read the completion message

**Expected behavior:**
- Status persists for 1-2 seconds then auto-clears
- User has time to read results

**Recommended Fix:**

Implement timeout-based clearing with timestamp:
```rust
use std::time::Instant;

pub struct AppState {
    pub status_message: Option<String>,
    pub status_message_timestamp: Option<Instant>,  // Add this
    // ... rest of fields ...
}

impl AppState {
    pub fn show_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_timestamp = Some(Instant::now());
    }
    
    pub fn clear_expired_status(&mut self) {
        if let Some(timestamp) = self.status_message_timestamp {
            if timestamp.elapsed() > Duration::from_secs(2) {
                self.status_message = None;
                self.status_message_timestamp = None;
            }
        }
    }
}

// In handle_key: REMOVE the auto-clear
fn handle_key(state: &mut AppState, key: KeyCode, viewport_height: usize) -> bool {
    // REMOVED: state.status_message = None;
    match &state.mode { ... }
}

// In render loop: auto-clear after 2 seconds
while !should_quit {
    // ... handle events ...
    state.clear_expired_status();
    terminal.draw(|frame| { ... })?;
}
```

---

## Medium-Priority Issues (Follow-Up PRs)

### 🟡 MEDIUM #6: Integer Overflow in Size Calculation

**Status:** ❌ UNSOLVED  
**Location:** `src/scanner/size.rs`  
**Severity:** MEDIUM  
**Effort:** 10 minutes

**Problem:** Size accumulation uses `u64` (max ~18.4 EB) without overflow checking.

**Risk:** Very low in practice (modern disks ~100TB), but theoretically unsound.

**Fix:** Use `checked_add()` with error handling.

---

### 🟡 MEDIUM #7: Hardcoded SKIP_DIRS Not Configurable

**Status:** ❌ UNSOLVED  
**Location:** `src/scanner/dir.rs:5`  
**Severity:** MEDIUM  
**Effort:** 15 minutes

**Problem:**
```rust
const SKIP_DIRS: &[&str] = &[".git", ".svn", ".hg", "node_modules/.cache", ".cache"];
```

Can't override `.git`, `.svn`, etc. even with `--exclude` flag. What if user wants to scan inside version control dirs for a size audit?

**Fix:** Add `--no-skip-hidden` flag or merge with `--exclude` logic.

---

### 🟡 MEDIUM #8: Background Thread Termination Is Silent

**Status:** ❌ UNSOLVED  
**Location:** `src/ui/tui.rs:140-144`  
**Severity:** MEDIUM  
**Effort:** 5 minutes

**Problem:** Thread exits with no logging if receiver drops.

**Impact:** Hard to debug if thread exits unexpectedly.

**Fix:** Add `eprintln!()` to log reason.

---

### 🟡 MEDIUM #9: Unused Parameter in Render Function

**Status:** ❌ UNSOLVED  
**Location:** `src/ui/tui.rs:251`  
**Severity:** MEDIUM  
**Effort:** 5 minutes

**Problem:** `_viewport_height` parameter is accepted but never used.

**Fix:** Remove if unused, or implement if future optimization planned.

---

### 🟡 MEDIUM #10: Mouse Click Bounds Checking Incomplete

**Status:** ❌ UNSOLVED  
**Location:** `src/ui/tui.rs:433-434`  
**Severity:** MEDIUM  
**Effort:** 15 minutes

**Problem:**
```rust
let visible_row = list_row + state.scroll_offset;  // ← No bounds check before indexing
```

**Risk:** Index out of bounds panic if clicking below visible list.

**Fix:** Add bounds check before using `visible_row`.

---

### 🟡 MEDIUM #11: Duplicate Ecosystem Filtering Logic

**Status:** ❌ UNSOLVED  
**Location:** `src/args.rs:94-110` + `src/ui/tui.rs:115-127`  
**Severity:** MEDIUM  
**Effort:** 20 minutes

**Problem:** Same filtering logic in two places.

**Impact:** Changes must be mirrored; risk of divergence.

**Fix:** Extract to shared function or call `args.get_ecosystems()` instead of reimplementing.

*(Also mentioned in CRITICAL #2)*

---

### 🟡 MEDIUM #12: No Validation of Ecosystem Configuration Files

**Status:** ❌ UNSOLVED  
**Location:** `src/config/ecosystem.rs:95-107`  
**Severity:** MEDIUM  
**Effort:** 15 minutes

**Problem:**

If all ecosystem JSON files are corrupt, app runs with 0 ecosystems (no error to user).

```rust
pub fn load_ecosystems() -> anyhow::Result<Vec<Ecosystem>> {
    let mut ecosystems = Vec::new();
    for entry in glob::glob("ecosystems/*.json")? {
        match entry {
            Ok(path) => match load_ecosystem_from_path(&path) {
                Ok(eco) => ecosystems.push(eco),
                Err(e) => eprintln!("Warning: failed to load {:?}: {}", path, e),  // ← Just warns!
            },
        }
    }
    Ok(ecosystems)  // ← Could be empty!
}
```

**Fix:** Return error if no valid ecosystems loaded:
```rust
if ecosystems.is_empty() {
    return Err(anyhow!("No valid ecosystems found in ecosystems/"));
}
```

---

### 🟡 MEDIUM #13: Size Loading State Is Unclear

**Status:** ❌ UNSOLVED  
**Location:** `src/config/ecosystem.rs:84`  
**Severity:** MEDIUM  
**Effort:** 30 minutes

**Problem:** Folders show "…" during loading with no indication of progress.

**Impact:** Users don't know if app is frozen or just calculating.

**Fix:** Add `SizeState` enum instead of just `u64`:
```rust
pub enum SizeBytes {
    Unknown,
    Calculating,
    Known(u64),
}
```

---

### 🟡 MEDIUM #14: No Recovery From Partial Deletion

**Status:** ❌ UNSOLVED  
**Location:** `src/deleter.rs:24-65`  
**Severity:** MEDIUM  
**Effort:** 60 minutes

**Problem:**

If deletion fails midway (e.g., permission denied on subdir), folder is partially deleted but app reports success.

**Impact:** Folder partially deleted; next run might not find it; user doesn't know cleanup is incomplete.

**Fix:** Use trash crate or atomic moves instead of `remove_dir_all()`.

---

### 🟡 MEDIUM #15: Wrong Default for Confidence Enum

**Status:** ❌ UNSOLVED  
**Location:** `src/config/ecosystem.rs:8`  
**Severity:** MEDIUM  
**Effort:** 5 minutes

**Problem:**
```rust
#[derive(Default)]
pub enum Confidence {
    #[default]
    Certain,  // ← But should be Undetected!
    ...
}
```

When `DiscoveredFolder::new()` is called directly (in tests), confidence defaults to Certain. But folders are only "Certain" if they matched unambiguously.

**Fix:** Change default to `Undetected`.

---

### 🟡 MEDIUM #16: Unclear Default Ecosystem Behavior

**Status:** ❌ UNSOLVED  
**Location:** `src/args.rs:107-109`  
**Severity:** MEDIUM  
**Effort:** 15 minutes

**Problem:**

Documentation claims "use local patterns when no --target", but code returns ALL ecosystems when neither `--all` nor `--target` specified.

**Impact:** Ambiguous intended behavior; hard to maintain.

**Fix:** Clarify docs or change implementation to match intended design.

---

## Low-Priority Issues (Optional)

### 🟢 LOW #17-25: Nine Low-Priority Improvements

**Combined Effort:** ~3.7 hours

1. **Missing doc comments** (AppState, ScanEvent, Ecosystem) - 30 min
2. **No integration tests** for TUI (hard due to state machine) - 60 min
3. **Missing error context** in Result types - 15 min
4. **State mutation fragility** (ordering dependencies) - 30 min
5. **Platform support claims misleading** (README vs Windows reality) - 10 min
6. **Confidence algorithm undocumented** - 20 min
7. **Keybindings hardcoded** (should extract to constant) - 10 min
8. **Plus 2 additional minor items** - varies

---

## Epic Issues / Feature Gaps

### ✅ SOLVED: Subfolder Filtering Problem

**Status:** ✅ SOLVED  
**Solution:** Implemented in `src/scanner/dir.rs` using `discovered_prefixes` HashSet to track and filter descendant paths. When `./target` is detected, its subfolders like `./target/build/` are not listed separately.

---

### ❌ UNSOLVED: Ecosystem Detection Accuracy

**Status:** ❌ UNSOLVED  
**Severity:** HIGH (Product Quality)  
**Effort:** Variable (requires research)

**Problem:** Need more intelligent detection of programming language ecosystems.

**Recommended Action:**
- Go through all files in `ecosystems/` directory
- Add relevant marker files to each ecosystem JSON for disambiguation
- Examples: `package.json` for Node.js, `Cargo.toml` for Rust, `go.mod` for Go
- Test marker-based detection to reduce ambiguity

**Current gap:** Many ecosystems have ambiguous folder names (e.g., "target" could be Rust or generic).

---

### ❌ UNSOLVED: Logo Animation

**Status:** ❌ UNSOLVED  
**Severity:** LOW (Polish)  
**Effort:** TBD

**Problem:** Logo animation for splash screen or loading state.

**Note:** Considered out of scope for initial v1.0 release.

---

### ❌ UNSOLVED: GitHub CI/CD Pipeline

**Status:** ❌ UNSOLVED  
**Severity:** CRITICAL (Release blocking)  
**Effort:** TBD (2-4 hours estimated)

**Requirements:**
- Run tests on push (cargo test)
- Run clippy on push (cargo clippy)
- Compile on multiple platforms: Linux, macOS, Windows
- Build release binaries for distribution
- Publish to crates.io automatically on tag

**Why blocking:** Without CI, we can't catch platform-specific issues (like CRITICAL #1 on Windows).

**Recommended:** Use GitHub Actions with matrix builds for Linux/macOS/Windows.

---

### ❌ UNSOLVED: Building Binary for Unix/macOS

**Status:** ❌ UNSOLVED  
**Severity:** CRITICAL (Release blocking)  
**Effort:** TBD (2-4 hours estimated)

**Requirements:**
- Build optimized release binary
- Create tar.gz distribution package
- Sign binaries (optional but recommended)
- Publish to GitHub Releases
- Consider Homebrew formula for easy installation

**Roadmap:** cargo-dist or cargo-binstall integration.

---

### ❌ UNSOLVED: Building Binary for Windows

**Status:** ❌ UNSOLVED  
**Severity:** CRITICAL (Release blocking - CRITICAL #1 must be fixed first)  
**Effort:** TBD (2-4 hours estimated)

**Requirements:**
- First fix CRITICAL #1 (Windows compilation broken)
- Build optimized release binary (.exe)
- Create Windows installer (optional)
- Publish to GitHub Releases
- Consider winget/scoop package managers

**Blocker:** CRITICAL #1 - Windows code won't compile until inode code is fixed.

---

### ❌ UNSOLVED: Binary Distribution Across Ecosystems

**Status:** ❌ UNSOLVED  
**Severity:** MEDIUM (Long-term)  
**Effort:** TBD (4-8 hours estimated)

**Question:** Can we publish to npm, PyPI, or other package ecosystems?

**Considerations:**
- Rust binary distribution via npm is possible but unconventional
- Would require Node.js wrapper (additional complexity)
- More natural: publish to crates.io, Homebrew, GitHub Releases only
- npm publication adds little value for a CLI tool

**Recommendation:** Focus on standard distribution methods (GitHub Releases, crates.io, Homebrew).

---

### ❌ UNSOLVED: Marketing Page + README.md Glow-up

**Status:** ❌ UNSOLVED  
**Severity:** MEDIUM (Before public release)  
**Effort:** TBD (4-8 hours estimated)

**Current state:**
- README is comprehensive but could be more engaging
- No dedicated marketing/landing page
- No installation instructions for different platforms
- CRITICAL #1 mentions: README claims "Windows support" but code doesn't compile on Windows

**Recommendations:**
1. Update README with Windows CI status
2. Add quick-start guide with screenshots
3. Add comparison to npkill and other tools
4. Create landing page (GitHub Pages or separate domain)
5. Add demo/animation of TUI in action
6. Clarify platform support after fixing CRITICAL #1

---
