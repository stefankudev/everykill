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

## Medium-Priority Issues (Follow-Up PRs)

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
