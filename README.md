# everykill

The CLI tool to find and delete dependency folders across all languages and ecosystems. Similar to npkill but supports **all** ecosystems, not just Node.js.

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run in development
cargo run

# Release build
cargo build --release

# Run tests
cargo test
```

## Usage

```bash
# Scan current directory
cargo run

# Scan specific directory
cargo run -- -d ~/projects

# Target specific ecosystems
cargo run -- -t nodejs,rust

# Include global caches
cargo run -- -t nodejs --global

# Exclude directories
cargo run -- -E "target,vendor"

# Sort by size (largest first)
cargo run -- -s size

# Don't scan subdirectories
cargo run -- --no-recursive

# Full help
cargo run -- --help
```

## Features

- **Multi-ecosystem**: Supports 60+ ecosystems (Node.js, Rust, Python, Go, Java, etc.)
- **Fast**: Parallel directory scanning with rayon
- **Flexible filtering**: Target specific ecosystems, exclude directories
- **Human-readable sizes**: Automatic unit formatting (MB, GB, etc.)
- **Safe**: No deletion by default (dry-run behavior)

## Architecture

```
src/
├── main.rs              # Binary entry point
├── lib.rs              # Library entry, calls run()
├── args.rs             # CLI argument parsing
├── config/             # Configuration & ecosystem loading
├── scanner/            # Directory scanning & size calculation
├── size_util.rs        # Size formatting utility
└── ui/                 # TUI components (planned)
```
