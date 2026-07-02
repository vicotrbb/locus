//! Parsers for Linux NUMA locality evidence.

use std::collections::BTreeMap;
use std::fmt;

use locus_core::NodeId;

/// One parsed mapping from `/proc/<pid>/numa_maps`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaMapsEntry {
    /// Mapping start address.
    pub start_address: u64,
    /// Kernel-reported policy token, such as `default`, `bind`, or `interleave`.
    pub policy: String,
    /// Per-node page counts from `N0=...` fields.
    pub node_pages: BTreeMap<NodeId, u64>,
    /// Other key-value attributes, such as `mapped=...` or `kernelpagesize_kB=...`.
    pub attributes: BTreeMap<String, String>,
    /// Flag-style attributes without an equals sign.
    pub flags: Vec<String>,
}

/// One parsed entry from cgroup v2 `memory.numa_stat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgroupNumaStatEntry {
    /// Metric name, such as `anon`, `file`, or `kernel_stack`.
    pub metric: String,
    /// Per-node byte counts from `N0=...` fields.
    pub node_bytes: BTreeMap<NodeId, u64>,
}

/// One parsed metric from `/sys/devices/system/node/node*/numastat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeNumastatMetric {
    /// Metric name, such as `numa_hit` or `other_node`.
    pub metric: String,
    /// Metric value.
    pub value: u64,
}

/// Parses all non-empty lines from `/proc/<pid>/numa_maps`.
///
/// # Errors
///
/// Returns an error when any line has an invalid address, missing policy, or
/// malformed per-node count.
pub fn parse_numa_maps(input: &str) -> Result<Vec<NumaMapsEntry>, ObserveParseError> {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            parse_numa_maps_line(line).map_err(|source| ObserveParseError::Line {
                line: index + 1,
                source: Box::new(source),
            })
        })
        .collect()
}

/// Parses one line from `/proc/<pid>/numa_maps`.
///
/// # Errors
///
/// Returns an error when the line has an invalid address, missing policy, or
/// malformed per-node count.
pub fn parse_numa_maps_line(line: &str) -> Result<NumaMapsEntry, ObserveParseError> {
    let mut tokens = line.split_whitespace();
    let address = tokens.next().ok_or(ObserveParseError::MissingAddress)?;
    let policy = tokens.next().ok_or(ObserveParseError::MissingPolicy)?;
    let start_address = u64::from_str_radix(address, 16)
        .map_err(|_| ObserveParseError::InvalidAddress(address.to_owned()))?;

    let mut node_pages = BTreeMap::new();
    let mut attributes = BTreeMap::new();
    let mut flags = Vec::new();

    for token in tokens {
        if let Some((key, value)) = token.split_once('=') {
            if let Some(node) = parse_node_key(key)? {
                let pages = parse_u64(value, token)?;
                node_pages.insert(node, pages);
            } else {
                attributes.insert(key.to_owned(), value.to_owned());
            }
        } else {
            flags.push(token.to_owned());
        }
    }

    Ok(NumaMapsEntry {
        start_address,
        policy: policy.to_owned(),
        node_pages,
        attributes,
        flags,
    })
}

/// Parses all non-empty lines from cgroup v2 `memory.numa_stat`.
///
/// # Errors
///
/// Returns an error when any line is missing a metric name or contains a
/// malformed per-node byte count.
pub fn parse_cgroup_numa_stat(input: &str) -> Result<Vec<CgroupNumaStatEntry>, ObserveParseError> {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            parse_cgroup_numa_stat_line(line).map_err(|source| ObserveParseError::Line {
                line: index + 1,
                source: Box::new(source),
            })
        })
        .collect()
}

/// Parses one cgroup v2 `memory.numa_stat` line.
///
/// # Errors
///
/// Returns an error when the line is missing a metric name or contains a
/// malformed per-node byte count.
pub fn parse_cgroup_numa_stat_line(line: &str) -> Result<CgroupNumaStatEntry, ObserveParseError> {
    let mut tokens = line.split_whitespace();
    let metric = tokens.next().ok_or(ObserveParseError::MissingMetric)?;
    let mut node_bytes = BTreeMap::new();

    for token in tokens {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| ObserveParseError::InvalidToken(token.to_owned()))?;
        let node = parse_node_key(key)?
            .ok_or_else(|| ObserveParseError::InvalidNodeKey(key.to_owned()))?;
        node_bytes.insert(node, parse_u64(value, token)?);
    }

    Ok(CgroupNumaStatEntry {
        metric: metric.to_owned(),
        node_bytes,
    })
}

/// Parses `/sys/devices/system/node/node*/numastat` content.
///
/// # Errors
///
/// Returns an error when any line is missing a metric value or has a non-numeric
/// value.
pub fn parse_node_numastat(input: &str) -> Result<Vec<NodeNumastatMetric>, ObserveParseError> {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            parse_node_numastat_line(line).map_err(|source| ObserveParseError::Line {
                line: index + 1,
                source: Box::new(source),
            })
        })
        .collect()
}

/// Parses one `/sys/devices/system/node/node*/numastat` line.
///
/// # Errors
///
/// Returns an error when the line is missing a metric value or has a non-numeric
/// value.
pub fn parse_node_numastat_line(line: &str) -> Result<NodeNumastatMetric, ObserveParseError> {
    let mut tokens = line.split_whitespace();
    let metric = tokens.next().ok_or(ObserveParseError::MissingMetric)?;
    let value = tokens
        .next()
        .ok_or_else(|| ObserveParseError::InvalidToken(line.to_owned()))?;

    Ok(NodeNumastatMetric {
        metric: metric.to_owned(),
        value: parse_u64(value, line)?,
    })
}

/// Parser errors for Linux NUMA observability files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObserveParseError {
    /// A nested parser failed at a specific line.
    Line {
        /// One-based line number.
        line: usize,
        /// Source error.
        source: Box<ObserveParseError>,
    },
    /// A `numa_maps` line did not include an address.
    MissingAddress,
    /// A `numa_maps` line did not include a policy token.
    MissingPolicy,
    /// A metric line did not include a metric name.
    MissingMetric,
    /// A mapping address was not hexadecimal.
    InvalidAddress(String),
    /// A token was malformed for its expected file format.
    InvalidToken(String),
    /// A node key was malformed.
    InvalidNodeKey(String),
    /// A numeric value was malformed.
    InvalidNumber {
        /// Original token.
        token: String,
        /// Invalid value.
        value: String,
    },
}

impl fmt::Display for ObserveParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line { line, source } => write!(f, "line {line}: {source}"),
            Self::MissingAddress => f.write_str("missing numa_maps address"),
            Self::MissingPolicy => f.write_str("missing numa_maps policy"),
            Self::MissingMetric => f.write_str("missing NUMA metric"),
            Self::InvalidAddress(value) => write!(f, "invalid mapping address: {value}"),
            Self::InvalidToken(token) => write!(f, "invalid token: {token}"),
            Self::InvalidNodeKey(key) => write!(f, "invalid NUMA node key: {key}"),
            Self::InvalidNumber { token, value } => {
                write!(f, "invalid number {value} in token {token}")
            }
        }
    }
}

impl std::error::Error for ObserveParseError {}

fn parse_node_key(key: &str) -> Result<Option<NodeId>, ObserveParseError> {
    let Some(rest) = key.strip_prefix('N') else {
        return Ok(None);
    };

    if rest.is_empty() || !rest.chars().all(|value| value.is_ascii_digit()) {
        return Err(ObserveParseError::InvalidNodeKey(key.to_owned()));
    }

    let parsed = rest
        .parse::<u32>()
        .map_err(|_| ObserveParseError::InvalidNodeKey(key.to_owned()))?;
    Ok(Some(NodeId(parsed)))
}

fn parse_u64(value: &str, token: &str) -> Result<u64, ObserveParseError> {
    value
        .parse::<u64>()
        .map_err(|_| ObserveParseError::InvalidNumber {
            token: token.to_owned(),
            value: value.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use locus_core::NodeId;

    use super::{
        parse_cgroup_numa_stat, parse_node_numastat, parse_numa_maps_line, ObserveParseError,
    };

    #[test]
    fn parses_numa_maps_line_with_nodes_and_attributes() {
        let entry = parse_numa_maps_line(
            "7f6c00000000 bind:0 file=/tmp/locus mapped=4 active=2 N0=1 N1=3 kernelpagesize_kB=4",
        )
        .expect("valid numa_maps line");

        assert_eq!(entry.start_address, 0x7f6c_0000_0000);
        assert_eq!(entry.policy, "bind:0");
        assert_eq!(entry.node_pages.get(&NodeId(0)), Some(&1));
        assert_eq!(entry.node_pages.get(&NodeId(1)), Some(&3));
        assert_eq!(
            entry.attributes.get("kernelpagesize_kB"),
            Some(&"4".to_owned())
        );
    }

    #[test]
    fn parses_cgroup_numa_stat_lines() {
        let entries = parse_cgroup_numa_stat("anon N0=4096 N1=8192\nfile N0=0 N1=1024\n")
            .expect("valid memory.numa_stat content");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].metric, "anon");
        assert_eq!(entries[0].node_bytes.get(&NodeId(1)), Some(&8192));
        assert_eq!(entries[1].metric, "file");
    }

    #[test]
    fn parses_node_numastat_metrics() {
        let metrics = parse_node_numastat("numa_hit 10\nnuma_miss 2\nother_node 1\n")
            .expect("valid node numastat content");

        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0].metric, "numa_hit");
        assert_eq!(metrics[1].value, 2);
    }

    #[test]
    fn reports_line_context_for_invalid_node_keys() {
        let error = parse_cgroup_numa_stat("anon N0=1\nfile NX=2\n")
            .expect_err("invalid node key should fail");

        assert_eq!(
            error,
            ObserveParseError::Line {
                line: 2,
                source: Box::new(ObserveParseError::InvalidNodeKey("NX".to_owned())),
            }
        );
    }
}
