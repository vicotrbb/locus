//! Linux CPU-list parsing and formatting.

use std::fmt;

/// A sorted, deduplicated set of CPU indexes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CpuSet {
    cpus: Vec<usize>,
}

impl CpuSet {
    /// Builds a CPU set from arbitrary CPU indexes.
    #[must_use]
    pub fn from_cpus(mut cpus: Vec<usize>) -> Self {
        cpus.sort_unstable();
        cpus.dedup();
        Self { cpus }
    }

    /// Parses Linux CPU-list syntax, such as `0-3,8,10-11`.
    ///
    /// Empty or whitespace-only input is accepted as an empty set because some
    /// sysfs attributes are optional or absent in virtualized environments.
    ///
    /// # Errors
    ///
    /// Returns an error when a segment is empty, a CPU index is not numeric, or
    /// a range is descending.
    pub fn parse_linux_list(input: &str) -> Result<Self, CpuSetParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(Self::default());
        }

        let mut cpus = Vec::new();
        for raw_part in trimmed.split(',') {
            let part = raw_part.trim();
            if part.is_empty() {
                return Err(CpuSetParseError::EmptySegment);
            }

            if let Some((start, end)) = part.split_once('-') {
                let start = parse_cpu(start)?;
                let end = parse_cpu(end)?;
                if start > end {
                    return Err(CpuSetParseError::DescendingRange { start, end });
                }
                cpus.extend(start..=end);
            } else {
                cpus.push(parse_cpu(part)?);
            }
        }

        Ok(Self::from_cpus(cpus))
    }

    /// Returns true when the set contains no CPUs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cpus.is_empty()
    }

    /// Returns the number of CPUs in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cpus.len()
    }

    /// Returns true when `cpu` is present.
    #[must_use]
    pub fn contains(&self, cpu: usize) -> bool {
        self.cpus.binary_search(&cpu).is_ok()
    }

    /// Iterates over CPUs in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.cpus.iter().copied()
    }

    /// Returns the set as a sorted vector.
    #[must_use]
    pub fn to_vec(&self) -> Vec<usize> {
        self.cpus.clone()
    }
}

impl fmt::Display for CpuSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, cpu) in self.cpus.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            write!(f, "{cpu}")?;
        }
        Ok(())
    }
}

/// Errors produced while parsing Linux CPU-list syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuSetParseError {
    /// A comma-separated segment was empty.
    EmptySegment,
    /// A segment was not an integer CPU index.
    InvalidCpu(String),
    /// A range had a start greater than its end.
    DescendingRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
    },
}

impl fmt::Display for CpuSetParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySegment => f.write_str("empty CPU-list segment"),
            Self::InvalidCpu(value) => write!(f, "invalid CPU index: {value}"),
            Self::DescendingRange { start, end } => {
                write!(f, "CPU range starts after it ends: {start}-{end}")
            }
        }
    }
}

impl std::error::Error for CpuSetParseError {}

fn parse_cpu(value: &str) -> Result<usize, CpuSetParseError> {
    value
        .trim()
        .parse::<usize>()
        .map_err(|_| CpuSetParseError::InvalidCpu(value.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{CpuSet, CpuSetParseError};

    #[test]
    fn parses_linux_cpulist_ranges_and_singles() {
        let set = CpuSet::parse_linux_list("0-3,8,10-11").expect("valid CPU list");

        assert_eq!(set.to_vec(), vec![0, 1, 2, 3, 8, 10, 11]);
        assert!(set.contains(8));
        assert!(!set.contains(9));
    }

    #[test]
    fn sorts_and_deduplicates_inputs() {
        let set = CpuSet::parse_linux_list("4,2,2,3-4").expect("valid CPU list");

        assert_eq!(set.to_vec(), vec![2, 3, 4]);
    }

    #[test]
    fn rejects_descending_ranges() {
        let error = CpuSet::parse_linux_list("3-1").expect_err("descending range should fail");

        assert_eq!(
            error,
            CpuSetParseError::DescendingRange { start: 3, end: 1 }
        );
    }
}
