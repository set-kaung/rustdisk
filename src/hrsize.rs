#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HumanReadableSize(pub u64);

fn convert(size: u64) -> String {
    let gigabyte = (1024 as u64).pow(3);
    let megabyte = (1024 as u64).pow(2);
    let kilobyte = (1024 as u64).pow(1);
    if size >= gigabyte {
        format!("{:10.3} GB", size as f64 / gigabyte as f64)
    } else if size >= megabyte {
        format!("{:10.3} MB", size as f64 / megabyte as f64)
    } else if size >= kilobyte {
        format!("{:10.3} KB", size as f64 / kilobyte as f64)
    } else {
        format!("{} B", size)
    }
}

impl std::fmt::Display for HumanReadableSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", convert(self.0))
    }
}

impl std::ops::AddAssign<u64> for HumanReadableSize {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}

impl Into<u64> for HumanReadableSize {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<f64> for HumanReadableSize {
    fn into(self) -> f64 {
        self.0 as f64
    }
}
impl From<u64> for HumanReadableSize {
    fn from(value: u64) -> Self {
        HumanReadableSize(value)
    }
}

impl From<&HumanReadableSize> for f64 {
    fn from(hr: &HumanReadableSize) -> Self {
        hr.0 as f64
    }
}
