# Size Utility Implementation Plan

## Overview

Create a size formatting utility (`src/size_util.rs`) that converts byte counts into human-readable strings using the most appropriate unit of measurement. The utility must handle the full range of `u64` (up to ~16 exabytes) and be thoroughly tested.

## Why This Utility Is Needed

Currently, folder sizes are displayed as raw byte counts:
```
./target (Rust) - 319576175 bytes
./node_modules (Node.js) - 15728640000 bytes
```

With the utility:
```
./target (Rust) - 305 MB
./node_modules (Node.js) - 14.6 GB
```

This improves readability significantly, especially for large dependency folders.

## Unit of Measurement Support

We use binary (1024-based) units since they are standard for file sizes:

| Unit | Symbol | Bytes | Range |
|------|--------|-------|-------|
| Byte | B | 1 | 0 to 1023 |
| Kilobyte | KB | 1024^1 | 1 KiB to 1023 KiB |
| Megabyte | MB | 1024^2 | 1 MiB to 1023 MiB |
| Gigabyte | GB | 1024^3 | 1 GiB to 1023 GiB |
| Terabyte | TB | 1024^4 | 1 TiB to 1023 TiB |
| Petabyte | PB | 1024^5 | 1 PiB to 1023 PiB |
| Exabyte | EB | 1024^6 | 1 EiB to ~16 EiB (max u64) |

**Note:** `u64::MAX` ≈ 18.4 quintillion bytes ≈ 16 exabytes, so EB is the maximum practical unit.

## Implementation Design

### File Location
```
src/size_util.rs    # New file
src/lib.rs          # Add pub mod size_util
```

### Core Function Signature

```rust
/// Format a byte count in the most appropriate unit
/// Returns a string like "305 MB" or "1.5 GB"
pub fn format_size(bytes: u64) -> String;

pub enum SizeUnit {
    B, KB, MB, GB, TB, PB, EB,
}

pub fn format_size_with_unit(bytes: u64) -> (f64, SizeUnit);
```

### Algorithm

**Approach:** Simple iterative division (no floats, no logs - optimal performance)

```rust
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    const DIVISOR: u64 = 1024;
    
    if bytes < DIVISOR {
        return format!("{} B", bytes);
    }
    
    let mut value = bytes as f64;
    let mut unit_index = 0;
    
    while value >= DIVISOR as f64 && unit_index < UNITS.len() - 1 {
        value /= DIVISOR as f64;
        unit_index += 1;
    }
    
    // Format with appropriate precision
    if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit_index])
    } else if value >= 10.0 {
        format!("{:.1} {}", value, UNITS[unit_index])
    } else {
        format!("{:.2} {}", value, UNITS[unit_index])
    }
}
```

### Precision Strategy

| Value Range | Format | Example |
|-------------|--------|---------|
| >= 100 | No decimals | `305 MB` |
| >= 10 | 1 decimal | `14.6 GB` |
| < 10 | 2 decimals | `1.23 MB` |

This ensures:
- Large values are clean integers (easy to read)
- Medium values show reasonable precision
- Small values show more precision (useful for small folders)

### Alternative: Pure Integer Approach (For Maximum Performance)

If we want to avoid floats entirely:

```rust
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    const DIVISOR: u64 = 1024;
    
    if bytes < DIVISOR {
        return format!("{} B", bytes);
    }
    
    let mut remainder = bytes;
    let mut unit_index = 0;
    
    loop {
        let next = remainder / DIVISOR;
        if next < DIVISOR || unit_index == UNITS.len() - 1 {
            let quotient = remainder / DIVISOR;
            let rem = remainder % DIVISOR;
            // Calculate first 2 significant digits of remainder
            // to provide meaningful decimal representation
            return format!("{}.{:02} {}", quotient, (rem * 100 / DIVISOR), UNITS[unit_index + 1]);
        }
        remainder = next;
        unit_index += 1;
    }
}
```

**Decision:** Use the float approach for simplicity and readability. The performance difference is negligible for the size formatting use case.

## Module Exports (`src/size_util.rs`)

```rust
use std::fmt;

/// Size units in order from smallest to largest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeUnit {
    B, KB, MB, GB, TB, PB, EB,
}

impl SizeUnit {
    pub fn symbol(&self) -> &'static str {
        match self {
            SizeUnit::B => "B",
            SizeUnit::KB => "KB",
            SizeUnit::MB => "MB",
            SizeUnit::GB => "GB",
            SizeUnit::TB => "TB",
            SizeUnit::PB => "PB",
            SizeUnit::EB => "EB",
        }
    }
}

impl fmt::Display for SizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

const DIVISOR: u64 = 1024;
const UNITS: &[SizeUnit] = &[
    SizeUnit::B, SizeUnit::KB, SizeUnit::MB, SizeUnit::GB,
    SizeUnit::TB, SizeUnit::PB, SizeUnit::EB,
];

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub value: f64,
    pub unit: SizeUnit,
}

impl Size {
    pub fn new(bytes: u64) -> Self {
        if bytes < DIVISOR {
            return Size { value: bytes as f64, unit: SizeUnit::B };
        }
        
        let mut value = bytes as f64;
        let mut unit_index = 0;
        
        while value >= DIVISOR as f64 && unit_index < UNITS.len() - 1 {
            value /= DIVISOR as f64;
            unit_index += 1;
        }
        
        Size { value, unit: UNITS[unit_index] }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value >= 100.0 {
            write!(f, "{:.0} {}", self.value, self.unit)
        } else if self.value >= 10.0 {
            write!(f, "{:.1} {}", self.value, self.unit)
        } else {
            write!(f, "{:.2} {}", self.value, self.unit)
        }
    }
}

/// Format bytes in the most appropriate unit
/// Example: format_size(1572864000) -> "1.50 GB"
pub fn format_size(bytes: u64) -> String {
    Size::new(bytes).to_string()
}

/// Get the value and unit separately for programmatic use
pub fn get_size(bytes: u64) -> Size {
    Size::new(bytes)
}
```

## Testing Strategy

### Comprehensive Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(10240), "10.0 KB");
        assert_eq!(format_size(102400), "100 KB");
        assert_eq!(format_size(1048575), "1024 KB"); // Note: rounds up
    }

    #[test]
    fn test_megabytes() {
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(15728640), "15.0 MB");
        assert_eq!(format_size(104857600), "100 MB");
        assert_eq!(format_size(1073741824), "1.00 GB"); // 1024^3
    }

    #[test]
    fn test_gigabytes() {
        assert_eq!(format_size(1073741824), "1.00 GB");
        assert_eq!(format_size(16106127360), "15.0 GB");
        assert_eq!(format_size(1099511627776), "1.00 TB"); // 1024^4
    }

    #[test]
    fn test_terabytes() {
        assert_eq!(format_size(1099511627776), "1.00 TB");
        assert_eq!(format_size(1125899906842624), "1.00 PB"); // 1024^5
    }

    #[test]
    fn test_petabytes() {
        assert_eq!(format_size(1125899906842624), "1.00 PB");
        assert_eq!(format_size(1152921504606846976), "1.00 EB"); // 1024^6
    }

    #[test]
    fn test_exabyte_boundary() {
        // u64::MAX is approximately 16 exabytes
        let max_u64 = u64::MAX;
        let size = get_size(max_u64);
        assert_eq!(size.unit, SizeUnit::EB);
        assert!(size.value < 20.0); // Should be around 16.something
    }

    #[test]
    fn test_precision_large_values() {
        // Values >= 100 should show no decimals
        assert_eq!(format_size(104857600), "100 MB");      // 100 MB
        assert_eq!(format_size(1048576000), "1.00 GB");    // Still 1.00, under 10
    }

    #[test]
    fn test_precision_medium_values() {
        // Values >= 10 and < 100 should show 1 decimal
        assert_eq!(format_size(15728640), "15.0 MB");     // 15 MB
        assert_eq!(format_size(51200), "50.0 KB");         // 50 KB
    }

    #[test]
    fn test_precision_small_values() {
        // Values < 10 should show 2 decimals
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_size_struct() {
        let size = Size::new(15728640);
        assert_eq!(size.unit, SizeUnit::MB);
        assert!((size.value - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_size_unit_symbols() {
        assert_eq!(SizeUnit::B.symbol(), "B");
        assert_eq!(SizeUnit::KB.symbol(), "KB");
        assert_eq!(SizeUnit::MB.symbol(), "MB");
        assert_eq!(SizeUnit::GB.symbol(), "GB");
        assert_eq!(SizeUnit::TB.symbol(), "TB");
        assert_eq!(SizeUnit::PB.symbol(), "PB");
        assert_eq!(SizeUnit::EB.symbol(), "EB");
    }

    #[test]
    fn test_unit_display() {
        assert_eq!(format!("{}", SizeUnit::KB), "KB");
        assert_eq!(format!("{}", SizeUnit::GB), "GB");
    }

    #[test]
    fn test_size_display() {
        assert_eq!(format!("{}", Size::new(0)), "0 B");
        assert_eq!(format!("{}", Size::new(1024)), "1.00 KB");
        assert_eq!(format!("{}", Size::new(104857600)), "100 MB");
    }
}
```

### Edge Case Tests

| Test | Value | Expected |
|------|-------|----------|
| Zero | 0 | "0 B" |
| Max u64 | 18,446,744,073,709,551,615 | "~16 EB" |
| Exactly 1 KB | 1024 | "1.00 KB" |
| Exactly 1 MB | 1,048,576 | "1.00 MB" |
| Exactly 1 GB | 1,073,741,824 | "1.00 GB" |
| Exactly 1 TB | 1,099,511,627,776 | "1.00 TB" |
| Exactly 1 PB | 1,125,899,906,842,624 | "1.00 PB" |
| Exactly 1 EB | 1,152,921,504,606,846,976 | "1.00 EB" |

## Integration

### Update `lib.rs`

```rust
pub mod size_util;

use size_util::format_size;

// In run():
println!("  {} ({}) - {}", folder.path.display(), folder.ecosystem, format_size(folder.size_bytes));
```

## Performance Considerations

1. **Float division only once** - Value is divided iteratively until correct unit is found, then formatted
2. **No allocation in hot path** - Returns formatted string (necessary anyway)
3. **Simple loop** - No log calculations, no complex math
4. **No HashMap lookups** - Match on small enum ( SizeUnit has 7 variants)

The utility will be called once per folder (typically < 100 folders), so performance is not critical. The focus is on correctness and readability.

## File Structure After Completion

```
src/
├── lib.rs
├── main.rs
├── config/
│   ├── mod.rs
│   └── ecosystem.rs
├── scanner/
│   ├── mod.rs
│   ├── dir.rs
│   └── size.rs
├── size_util.rs      # NEW
└── ui/
    ├── mod.rs
    └── ascii.rs
```

## Summary

| Aspect | Design |
|--------|--------|
| Algorithm | Iterative division by 1024 |
| Max unit | EB (Exabyte) |
| Precision | Dynamic: 0/1/2 decimals based on magnitude |
| Performance | O(1) - constant loop iterations (max 6) |
| Test coverage | 14+ unit tests covering all units and edge cases |
| Return type | String for display, Size struct for programmatic use |
