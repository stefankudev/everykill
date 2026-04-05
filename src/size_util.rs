use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeUnit {
    B,
    KB,
    MB,
    GB,
    TB,
    PB,
    EB,
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
    SizeUnit::B,
    SizeUnit::KB,
    SizeUnit::MB,
    SizeUnit::GB,
    SizeUnit::TB,
    SizeUnit::PB,
    SizeUnit::EB,
];

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub value: f64,
    pub unit: SizeUnit,
}

impl Size {
    pub fn new(bytes: u64) -> Self {
        if bytes < DIVISOR {
            return Size {
                value: bytes as f64,
                unit: SizeUnit::B,
            };
        }

        let mut value = bytes as f64;
        let mut unit_index = 0;

        while value >= DIVISOR as f64 && unit_index < UNITS.len() - 1 {
            value /= DIVISOR as f64;
            unit_index += 1;
        }

        Size {
            value,
            unit: UNITS[unit_index],
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unit == SizeUnit::B {
            return write!(f, "{:.0} {}", self.value, self.unit);
        }
        if self.value == 0.0 {
            return write!(f, "0 B");
        }
        if self.value >= 100.0 {
            write!(f, "{:.0} {}", self.value, self.unit)
        } else if self.value >= 10.0 {
            write!(f, "{:.1} {}", self.value, self.unit)
        } else {
            write!(f, "{:.2} {}", self.value, self.unit)
        }
    }
}

pub fn format_size(bytes: u64) -> String {
    Size::new(bytes).to_string()
}

pub fn get_size(bytes: u64) -> Size {
    Size::new(bytes)
}

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
        assert_eq!(format_size(1048575), "1024 KB");
    }

    #[test]
    fn test_megabytes() {
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(15728640), "15.0 MB");
        assert_eq!(format_size(104857600), "100 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_gigabytes() {
        assert_eq!(format_size(1073741824), "1.00 GB");
        assert_eq!(format_size(16106127360), "15.0 GB");
        assert_eq!(format_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_terabytes() {
        assert_eq!(format_size(1099511627776), "1.00 TB");
        assert_eq!(format_size(1125899906842624), "1.00 PB");
    }

    #[test]
    fn test_petabytes() {
        assert_eq!(format_size(1125899906842624), "1.00 PB");
        assert_eq!(format_size(1152921504606846976), "1.00 EB");
    }

    #[test]
    fn test_exabyte_boundary() {
        let max_u64 = u64::MAX;
        let size = get_size(max_u64);
        assert_eq!(size.unit, SizeUnit::EB);
        assert!(size.value < 20.0);
    }

    #[test]
    fn test_precision_large_values() {
        assert_eq!(format_size(104857600), "100 MB");
        assert_eq!(format_size(1048576000), "1000 MB");
    }

    #[test]
    fn test_precision_medium_values() {
        assert_eq!(format_size(15728640), "15.0 MB");
        assert_eq!(format_size(51200), "50.0 KB");
    }

    #[test]
    fn test_precision_small_values() {
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
