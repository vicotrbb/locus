//! Parsers for Linux NUMA locality evidence.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

/// Snapshot of one node `numastat` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeNumastatSnapshot {
    /// Number of metrics inspected.
    pub metric_count: usize,
    /// Metric values by name.
    pub values: BTreeMap<String, u64>,
}

impl NodeNumastatSnapshot {
    /// Builds a snapshot from parsed node `numastat` metrics.
    #[must_use]
    pub fn from_metrics(metrics: &[NodeNumastatMetric]) -> Self {
        let mut values = BTreeMap::new();
        for metric in metrics {
            values.insert(metric.metric.clone(), metric.value);
        }

        Self {
            metric_count: values.len(),
            values,
        }
    }

    /// Returns a metric value by name.
    #[must_use]
    pub fn get(&self, metric: &str) -> Option<u64> {
        self.values.get(metric).copied()
    }

    /// Computes signed metric deltas from `previous` to `self`.
    #[must_use]
    pub fn delta_since(&self, previous: &Self) -> NodeNumastatDelta {
        let mut deltas = BTreeMap::new();

        for metric in previous.values.keys().chain(self.values.keys()) {
            let before = i128::from(previous.get(metric).unwrap_or_default());
            let after = i128::from(self.get(metric).unwrap_or_default());
            deltas.insert(metric.clone(), after - before);
        }

        NodeNumastatDelta { deltas }
    }
}

/// Signed delta between two node `numastat` snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeNumastatDelta {
    /// Delta values by metric name.
    pub deltas: BTreeMap<String, i128>,
}

impl NodeNumastatDelta {
    /// Returns a delta by metric name.
    #[must_use]
    pub fn get(&self, metric: &str) -> Option<i128> {
        self.deltas.get(metric).copied()
    }
}

/// Snapshot of node `numastat` metrics across NUMA nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeNumastatSystemSnapshot {
    /// Number of nodes inspected.
    pub node_count: usize,
    /// Per-node snapshots.
    pub nodes: BTreeMap<NodeId, NodeNumastatSnapshot>,
}

impl NodeNumastatSystemSnapshot {
    /// Builds a system snapshot from per-node snapshots.
    #[must_use]
    pub fn from_nodes(nodes: BTreeMap<NodeId, NodeNumastatSnapshot>) -> Self {
        Self {
            node_count: nodes.len(),
            nodes,
        }
    }

    /// Returns one node snapshot.
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&NodeNumastatSnapshot> {
        self.nodes.get(&node)
    }

    /// Computes signed per-node metric deltas from `previous` to `self`.
    #[must_use]
    pub fn delta_since(&self, previous: &Self) -> NodeNumastatSystemDelta {
        let mut nodes = BTreeMap::new();

        for node in previous.nodes.keys().chain(self.nodes.keys()) {
            let before = previous.nodes.get(node).cloned().unwrap_or_default();
            let after = self.nodes.get(node).cloned().unwrap_or_default();
            nodes.insert(*node, after.delta_since(&before));
        }

        NodeNumastatSystemDelta { nodes }
    }
}

/// Signed delta between two system node `numastat` snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeNumastatSystemDelta {
    /// Per-node metric deltas.
    pub nodes: BTreeMap<NodeId, NodeNumastatDelta>,
}

impl NodeNumastatSystemDelta {
    /// Returns one node delta.
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&NodeNumastatDelta> {
        self.nodes.get(&node)
    }
}

/// Summary of page placement derived from `numa_maps`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumaMapsSummary {
    /// Number of mappings inspected.
    pub mapping_count: usize,
    /// Total pages reported across all `N*=` fields.
    pub total_pages: u64,
    /// Per-node page totals.
    pub pages_by_node: BTreeMap<NodeId, u64>,
    /// Mapping counts by kernel policy token.
    pub mappings_by_policy: BTreeMap<String, usize>,
    /// Page totals by `kernelpagesize_kB` when present.
    pub pages_by_kernel_page_kb: BTreeMap<u64, u64>,
}

/// Classification of observed page placement for one mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPlacementStatus {
    /// The mapping entry did not report per-node pages yet.
    NoPagesReported,
    /// Every reported page is on the expected node.
    AllPagesOnExpectedNode,
    /// Some reported pages are on the expected node and some are elsewhere.
    PartialPagesOnExpectedNode,
    /// Reported pages exist, but none are on the expected node.
    NoPagesOnExpectedNode,
}

impl NumaPlacementStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoPagesReported => "no_pages_reported",
            Self::AllPagesOnExpectedNode => "all_pages_on_expected_node",
            Self::PartialPagesOnExpectedNode => "partial_pages_on_expected_node",
            Self::NoPagesOnExpectedNode => "no_pages_on_expected_node",
        }
    }
}

impl fmt::Display for NumaPlacementStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Placement evidence for one `numa_maps` entry against an expected node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaPlacementEvidence {
    /// Expected NUMA node.
    pub expected_node: NodeId,
    /// Total pages reported by the entry.
    pub total_pages: u64,
    /// Pages reported on the expected node.
    pub expected_node_pages: u64,
    /// Pages reported on nodes other than the expected node.
    pub other_node_pages: BTreeMap<NodeId, u64>,
    /// Placement classification.
    pub status: NumaPlacementStatus,
}

impl NumaMapsSummary {
    /// Builds a summary from parsed `numa_maps` entries.
    #[must_use]
    pub fn from_entries(entries: &[NumaMapsEntry]) -> Self {
        let mut summary = Self {
            mapping_count: entries.len(),
            ..Self::default()
        };

        for entry in entries {
            *summary
                .mappings_by_policy
                .entry(entry.policy.clone())
                .or_default() += 1;

            let entry_pages = entry
                .node_pages
                .values()
                .copied()
                .fold(0_u64, u64::saturating_add);
            if let Some(page_kb) = entry
                .attributes
                .get("kernelpagesize_kB")
                .and_then(|value| value.parse::<u64>().ok())
            {
                *summary.pages_by_kernel_page_kb.entry(page_kb).or_default() += entry_pages;
            }

            for (node, pages) in &entry.node_pages {
                summary.total_pages = summary.total_pages.saturating_add(*pages);
                *summary.pages_by_node.entry(*node).or_default() += pages;
            }
        }

        summary
    }
}

impl NumaPlacementEvidence {
    /// Builds placement evidence from one parsed `numa_maps` entry.
    #[must_use]
    pub fn from_entry(entry: &NumaMapsEntry, expected_node: NodeId) -> Self {
        let mut total_pages = 0_u64;
        let mut expected_node_pages = 0_u64;
        let mut other_node_pages = BTreeMap::new();

        for (node, pages) in &entry.node_pages {
            total_pages = total_pages.saturating_add(*pages);
            if *node == expected_node {
                expected_node_pages = expected_node_pages.saturating_add(*pages);
            } else {
                other_node_pages.insert(*node, *pages);
            }
        }

        let status = if total_pages == 0 {
            NumaPlacementStatus::NoPagesReported
        } else if expected_node_pages == total_pages {
            NumaPlacementStatus::AllPagesOnExpectedNode
        } else if expected_node_pages > 0 {
            NumaPlacementStatus::PartialPagesOnExpectedNode
        } else {
            NumaPlacementStatus::NoPagesOnExpectedNode
        };

        Self {
            expected_node,
            total_pages,
            expected_node_pages,
            other_node_pages,
            status,
        }
    }
}

/// Summary of cgroup NUMA bytes for one or more metrics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CgroupNumaSummary {
    /// Number of metric rows inspected.
    pub metric_count: usize,
    /// Total bytes reported across all node fields.
    pub total_bytes: u64,
    /// Per-node byte totals across metrics.
    pub bytes_by_node: BTreeMap<NodeId, u64>,
}

/// Signed delta between two cgroup NUMA summaries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CgroupNumaDelta {
    /// Signed total-byte delta across all node fields.
    pub total_bytes_delta: i128,
    /// Signed per-node byte deltas.
    pub bytes_by_node_delta: BTreeMap<NodeId, i128>,
}

impl CgroupNumaSummary {
    /// Builds a summary from parsed cgroup NUMA stat entries.
    #[must_use]
    pub fn from_entries(entries: &[CgroupNumaStatEntry]) -> Self {
        let mut summary = Self {
            metric_count: entries.len(),
            ..Self::default()
        };

        for entry in entries {
            for (node, bytes) in &entry.node_bytes {
                summary.total_bytes = summary.total_bytes.saturating_add(*bytes);
                *summary.bytes_by_node.entry(*node).or_default() += bytes;
            }
        }

        summary
    }

    /// Computes signed byte deltas from `previous` to `self`.
    #[must_use]
    pub fn delta_since(&self, previous: &Self) -> CgroupNumaDelta {
        let total_bytes_delta = i128::from(self.total_bytes) - i128::from(previous.total_bytes);
        let mut bytes_by_node_delta = BTreeMap::new();

        for node in previous
            .bytes_by_node
            .keys()
            .chain(self.bytes_by_node.keys())
        {
            let before = i128::from(
                previous
                    .bytes_by_node
                    .get(node)
                    .copied()
                    .unwrap_or_default(),
            );
            let after = i128::from(self.bytes_by_node.get(node).copied().unwrap_or_default());
            bytes_by_node_delta.insert(*node, after - before);
        }

        CgroupNumaDelta {
            total_bytes_delta,
            bytes_by_node_delta,
        }
    }
}

impl CgroupNumaDelta {
    /// Returns a signed delta for one node.
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<i128> {
        self.bytes_by_node_delta.get(&node).copied()
    }

    /// Returns true when any aggregate or node delta is non-zero.
    #[must_use]
    pub fn has_nonzero_delta(&self) -> bool {
        self.total_bytes_delta != 0 || self.bytes_by_node_delta.values().any(|delta| *delta != 0)
    }
}

/// Reads and parses a `numa_maps` file from an explicit path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsing fails.
pub fn read_numa_maps(path: impl AsRef<Path>) -> Result<Vec<NumaMapsEntry>, ObserveReadError> {
    let path = path.as_ref();
    let input = read_to_string(path)?;
    parse_numa_maps(&input).map_err(ObserveReadError::Parse)
}

/// Reads and parses `/proc/self/numa_maps`.
///
/// # Errors
///
/// Returns an error when `/proc/self/numa_maps` cannot be read or parsing
/// fails. Non-Linux hosts normally return a read error.
pub fn read_self_numa_maps() -> Result<Vec<NumaMapsEntry>, ObserveReadError> {
    read_numa_maps("/proc/self/numa_maps")
}

/// Finds the `numa_maps` entry with an exact start address.
#[must_use]
pub fn numa_maps_entry_by_start_address(
    entries: &[NumaMapsEntry],
    start_address: usize,
) -> Option<&NumaMapsEntry> {
    entries.iter().find(|entry| {
        usize::try_from(entry.start_address).is_ok_and(|entry_start| entry_start == start_address)
    })
}

/// Finds the ordered `numa_maps` entry containing an address.
///
/// This treats the next entry start as the end of the current range, matching
/// the address ordering used by `/proc/<pid>/numa_maps`.
#[must_use]
pub fn numa_maps_entry_containing_address(
    entries: &[NumaMapsEntry],
    address: usize,
) -> Option<&NumaMapsEntry> {
    let address = u64::try_from(address).ok()?;

    entries.iter().enumerate().find_map(|(index, entry)| {
        let next_start = entries.get(index + 1).map(|next| next.start_address);
        if entry.start_address <= address && next_start.map_or(true, |next| address < next) {
            Some(entry)
        } else {
            None
        }
    })
}

/// Reads and parses cgroup v2 `memory.numa_stat` from an explicit path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsing fails.
pub fn read_cgroup_numa_stat(
    path: impl AsRef<Path>,
) -> Result<Vec<CgroupNumaStatEntry>, ObserveReadError> {
    let path = path.as_ref();
    let input = read_to_string(path)?;
    parse_cgroup_numa_stat(&input).map_err(ObserveReadError::Parse)
}

/// Resolves the current cgroup v2 `memory.numa_stat` path from cgroup file
/// content.
///
/// # Errors
///
/// Returns an error when no unified cgroup v2 entry is present.
pub fn resolve_cgroup_v2_memory_numa_stat_path(
    cgroup_content: &str,
    cgroup_root: impl AsRef<Path>,
) -> Result<PathBuf, CgroupPathError> {
    let cgroup_root = cgroup_root.as_ref();
    for line in cgroup_content.lines() {
        let Some(rest) = line.strip_prefix("0::") else {
            continue;
        };
        let relative = rest.trim_start_matches('/');
        return Ok(cgroup_root.join(relative).join("memory.numa_stat"));
    }

    Err(CgroupPathError::MissingUnifiedEntry)
}

/// Reads and parses a node `numastat` file from an explicit path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsing fails.
pub fn read_node_numastat(
    path: impl AsRef<Path>,
) -> Result<Vec<NodeNumastatMetric>, ObserveReadError> {
    let path = path.as_ref();
    let input = read_to_string(path)?;
    parse_node_numastat(&input).map_err(ObserveReadError::Parse)
}

/// Reads per-node `numastat` files from a Linux node sysfs root.
///
/// # Errors
///
/// Returns an error when the root cannot be read or a discovered node
/// `numastat` file cannot be read or parsed. Missing `numastat` files for
/// individual node directories are skipped.
pub fn read_node_numastat_system_snapshot(
    node_root: impl AsRef<Path>,
) -> Result<NodeNumastatSystemSnapshot, ObserveReadError> {
    let node_root = node_root.as_ref();
    let entries = fs::read_dir(node_root).map_err(|source| ObserveReadError::Read {
        path: node_root.to_path_buf(),
        source,
    })?;
    let mut nodes = BTreeMap::new();

    for entry in entries {
        let entry = entry.map_err(|source| ObserveReadError::Read {
            path: node_root.to_path_buf(),
            source,
        })?;
        let Some(node) = parse_node_dir_name(&entry.file_name().to_string_lossy()) else {
            continue;
        };
        let stat_path = entry.path().join("numastat");
        match read_node_numastat(&stat_path) {
            Ok(metrics) => {
                nodes.insert(node, NodeNumastatSnapshot::from_metrics(&metrics));
            }
            Err(ObserveReadError::Read { source, .. })
                if source.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    Ok(NodeNumastatSystemSnapshot::from_nodes(nodes))
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

/// Read errors for Linux NUMA observability files.
#[derive(Debug)]
pub enum ObserveReadError {
    /// Filesystem read failed.
    Read {
        /// File path.
        path: PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// File content could not be parsed.
    Parse(ObserveParseError),
}

impl fmt::Display for ObserveReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Self::Parse(source) => write!(f, "failed to parse observability file: {source}"),
        }
    }
}

impl std::error::Error for ObserveReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse(source) => Some(source),
        }
    }
}

/// Cgroup path resolution failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgroupPathError {
    /// `/proc/self/cgroup` did not contain a unified cgroup v2 entry.
    MissingUnifiedEntry,
}

impl fmt::Display for CgroupPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingUnifiedEntry => f.write_str("missing unified cgroup v2 entry"),
        }
    }
}

impl std::error::Error for CgroupPathError {}

fn read_to_string(path: &Path) -> Result<String, ObserveReadError> {
    fs::read_to_string(path).map_err(|source| ObserveReadError::Read {
        path: path.to_path_buf(),
        source,
    })
}

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

fn parse_node_dir_name(name: &str) -> Option<NodeId> {
    let rest = name.strip_prefix("node")?;
    if rest.is_empty() || !rest.chars().all(|value| value.is_ascii_digit()) {
        return None;
    }

    rest.parse::<u32>().ok().map(NodeId)
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
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use locus_core::NodeId;
    use tempfile::TempDir;

    use super::{
        numa_maps_entry_by_start_address, numa_maps_entry_containing_address,
        parse_cgroup_numa_stat, parse_node_numastat, parse_numa_maps, parse_numa_maps_line,
        read_cgroup_numa_stat, read_node_numastat, read_node_numastat_system_snapshot,
        read_numa_maps, resolve_cgroup_v2_memory_numa_stat_path, CgroupNumaDelta,
        CgroupNumaSummary, CgroupPathError, NodeNumastatSnapshot, NodeNumastatSystemSnapshot,
        NumaMapsSummary, NumaPlacementEvidence, NumaPlacementStatus, ObserveParseError,
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
    fn finds_numa_maps_entry_by_start_address() {
        let entries = parse_numa_maps(
            "1000 default anon=1 N0=1\n\
             2000 bind:0 anon=2 N0=2\n",
        )
        .expect("valid numa maps");

        let entry = numa_maps_entry_by_start_address(&entries, 0x2000).expect("entry");

        assert_eq!(entry.policy, "bind:0");
        assert!(numa_maps_entry_by_start_address(&entries, 0x3000).is_none());
    }

    #[test]
    fn finds_numa_maps_entry_containing_address() {
        let entries = parse_numa_maps(
            "1000 default anon=1 N0=1\n\
             2000 bind:0 anon=2 N0=2\n\
             4000 default anon=1 N1=1\n",
        )
        .expect("valid numa maps");

        let entry = numa_maps_entry_containing_address(&entries, 0x2fff).expect("entry");

        assert_eq!(entry.policy, "bind:0");
        assert!(numa_maps_entry_containing_address(&entries, 0x0fff).is_none());
        assert_eq!(
            numa_maps_entry_containing_address(&entries, 0x4000)
                .expect("last entry")
                .policy,
            "default"
        );
    }

    #[test]
    fn resolves_cgroup_v2_memory_numa_stat_path() {
        let path = resolve_cgroup_v2_memory_numa_stat_path(
            "11:memory:/legacy\n0::/user.slice/session.scope\n",
            "/sys/fs/cgroup",
        )
        .expect("cgroup path");

        assert_eq!(
            path,
            PathBuf::from("/sys/fs/cgroup/user.slice/session.scope/memory.numa_stat")
        );
    }

    #[test]
    fn rejects_missing_cgroup_v2_entry() {
        let error =
            resolve_cgroup_v2_memory_numa_stat_path("11:memory:/legacy\n", "/sys/fs/cgroup")
                .expect_err("missing unified entry");

        assert_eq!(error, CgroupPathError::MissingUnifiedEntry);
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
    fn summarizes_node_numastat_metrics_and_deltas() {
        let before =
            parse_node_numastat("numa_hit 10\nnuma_miss 2\nother_node 1\n").expect("before");
        let after = parse_node_numastat("numa_hit 15\nnuma_miss 1\nlocal_node 7\n").expect("after");

        let before = NodeNumastatSnapshot::from_metrics(&before);
        let after = NodeNumastatSnapshot::from_metrics(&after);
        let delta = after.delta_since(&before);

        assert_eq!(before.metric_count, 3);
        assert_eq!(before.get("numa_hit"), Some(10));
        assert_eq!(delta.get("numa_hit"), Some(5));
        assert_eq!(delta.get("numa_miss"), Some(-1));
        assert_eq!(delta.get("other_node"), Some(-1));
        assert_eq!(delta.get("local_node"), Some(7));
    }

    #[test]
    fn summarizes_system_node_numastat_snapshots_and_deltas() {
        let before_node0 = NodeNumastatSnapshot::from_metrics(
            &parse_node_numastat("numa_hit 10\nnuma_miss 2\n").expect("node0 before"),
        );
        let before_node1 = NodeNumastatSnapshot::from_metrics(
            &parse_node_numastat("numa_hit 5\nother_node 1\n").expect("node1 before"),
        );
        let after_node0 = NodeNumastatSnapshot::from_metrics(
            &parse_node_numastat("numa_hit 15\nnuma_miss 1\n").expect("node0 after"),
        );
        let after_node2 = NodeNumastatSnapshot::from_metrics(
            &parse_node_numastat("local_node 7\n").expect("node2 after"),
        );

        let before = NodeNumastatSystemSnapshot::from_nodes(BTreeMap::from([
            (NodeId(0), before_node0),
            (NodeId(1), before_node1),
        ]));
        let after = NodeNumastatSystemSnapshot::from_nodes(BTreeMap::from([
            (NodeId(0), after_node0),
            (NodeId(2), after_node2),
        ]));
        let delta = after.delta_since(&before);

        assert_eq!(before.node_count, 2);
        assert_eq!(
            before.get(NodeId(1)).and_then(|node| node.get("numa_hit")),
            Some(5)
        );
        assert_eq!(
            delta.get(NodeId(0)).and_then(|node| node.get("numa_hit")),
            Some(5)
        );
        assert_eq!(
            delta.get(NodeId(0)).and_then(|node| node.get("numa_miss")),
            Some(-1)
        );
        assert_eq!(
            delta.get(NodeId(1)).and_then(|node| node.get("numa_hit")),
            Some(-5)
        );
        assert_eq!(
            delta.get(NodeId(2)).and_then(|node| node.get("local_node")),
            Some(7)
        );
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

    #[test]
    fn reads_observability_files_from_explicit_paths() {
        let temp = TempDir::new().expect("tempdir");
        let numa_maps = temp.path().join("numa_maps");
        let cgroup_stat = temp.path().join("memory.numa_stat");
        let node_stat = temp.path().join("numastat");

        fs::write(
            &numa_maps,
            "7f6c00000000 default anon=4 dirty=4 N0=4 kernelpagesize_kB=4\n",
        )
        .expect("write numa_maps");
        fs::write(&cgroup_stat, "anon N0=4096 N1=8192\n").expect("write cgroup stat");
        fs::write(&node_stat, "numa_hit 10\nother_node 1\n").expect("write node stat");

        assert_eq!(read_numa_maps(&numa_maps).expect("read numa maps").len(), 1);
        assert_eq!(
            read_cgroup_numa_stat(&cgroup_stat)
                .expect("read cgroup stat")
                .len(),
            1
        );
        assert_eq!(
            read_node_numastat(&node_stat)
                .expect("read node stat")
                .len(),
            2
        );
    }

    #[test]
    fn reads_node_numastat_system_snapshot_from_sysfs_root() {
        let temp = TempDir::new().expect("tempdir");
        let node0 = temp.path().join("node0");
        let node1 = temp.path().join("node1");
        let node_bad = temp.path().join("nodebad");

        fs::create_dir(&node0).expect("node0");
        fs::create_dir(&node1).expect("node1");
        fs::create_dir(&node_bad).expect("nodebad");
        fs::write(node0.join("numastat"), "numa_hit 10\nother_node 1\n").expect("node0 stat");
        fs::write(node1.join("numastat"), "numa_hit 20\nlocal_node 7\n").expect("node1 stat");
        fs::write(temp.path().join("online"), "0-1\n").expect("online");

        let snapshot = read_node_numastat_system_snapshot(temp.path()).expect("snapshot");

        assert_eq!(snapshot.node_count, 2);
        assert_eq!(
            snapshot
                .get(NodeId(0))
                .and_then(|node| node.get("numa_hit")),
            Some(10)
        );
        assert_eq!(
            snapshot
                .get(NodeId(1))
                .and_then(|node| node.get("local_node")),
            Some(7)
        );
    }

    #[test]
    fn summarizes_numa_maps_pages_by_node() {
        let entries = parse_numa_maps(
            "1000 default anon=3 N0=1 N1=2 kernelpagesize_kB=4\n\
             2000 bind:1 anon=4 dirty=4 N1=4 kernelpagesize_kB=2048\n\
             3000 default anon=1 N0=1 kernelpagesize_kB=4\n",
        )
        .expect("valid numa maps");

        let summary = NumaMapsSummary::from_entries(&entries);

        assert_eq!(summary.mapping_count, 3);
        assert_eq!(summary.total_pages, 8);
        assert_eq!(summary.pages_by_node.get(&NodeId(0)), Some(&2));
        assert_eq!(summary.pages_by_node.get(&NodeId(1)), Some(&6));
        assert_eq!(summary.mappings_by_policy.get("default"), Some(&2));
        assert_eq!(summary.mappings_by_policy.get("bind:1"), Some(&1));
        assert_eq!(summary.pages_by_kernel_page_kb.get(&4), Some(&4));
        assert_eq!(summary.pages_by_kernel_page_kb.get(&2048), Some(&4));
    }

    #[test]
    fn classifies_numa_placement_evidence() {
        let entries = parse_numa_maps(
            "1000 bind:0 anon=4 N0=4\n\
             2000 bind:0 anon=4 N0=1 N1=3\n\
             3000 bind:0 anon=4 N1=4\n\
             4000 default anon=0\n",
        )
        .expect("valid numa maps");

        let all_local = NumaPlacementEvidence::from_entry(&entries[0], NodeId(0));
        let partial = NumaPlacementEvidence::from_entry(&entries[1], NodeId(0));
        let remote = NumaPlacementEvidence::from_entry(&entries[2], NodeId(0));
        let no_pages = NumaPlacementEvidence::from_entry(&entries[3], NodeId(0));

        assert_eq!(
            all_local.status,
            NumaPlacementStatus::AllPagesOnExpectedNode
        );
        assert_eq!(all_local.total_pages, 4);
        assert_eq!(all_local.expected_node_pages, 4);
        assert_eq!(
            partial.status,
            NumaPlacementStatus::PartialPagesOnExpectedNode
        );
        assert_eq!(partial.other_node_pages.get(&NodeId(1)), Some(&3));
        assert_eq!(remote.status, NumaPlacementStatus::NoPagesOnExpectedNode);
        assert_eq!(no_pages.status, NumaPlacementStatus::NoPagesReported);
        assert_eq!(
            NumaPlacementStatus::AllPagesOnExpectedNode.to_string(),
            "all_pages_on_expected_node"
        );
    }

    #[test]
    fn summarizes_cgroup_numa_bytes_by_node() {
        let entries = parse_cgroup_numa_stat("anon N0=4096 N1=8192\nfile N0=10 N1=20\n")
            .expect("valid cgroup stats");

        let summary = CgroupNumaSummary::from_entries(&entries);

        assert_eq!(summary.metric_count, 2);
        assert_eq!(summary.total_bytes, 12_318);
        assert_eq!(summary.bytes_by_node.get(&NodeId(0)), Some(&4_106));
        assert_eq!(summary.bytes_by_node.get(&NodeId(1)), Some(&8_212));
    }

    #[test]
    fn computes_cgroup_numa_summary_delta() {
        let before = CgroupNumaSummary::from_entries(
            &parse_cgroup_numa_stat("anon N0=4096 N1=8192\nfile N0=100 N1=10\n").expect("before"),
        );
        let after = CgroupNumaSummary::from_entries(
            &parse_cgroup_numa_stat("anon N0=8192 N1=4096\nfile N0=100 N2=7\n").expect("after"),
        );

        let delta = after.delta_since(&before);
        let zero_delta = before.delta_since(&before);

        assert_eq!(
            delta,
            CgroupNumaDelta {
                total_bytes_delta: -3,
                bytes_by_node_delta: BTreeMap::from([
                    (NodeId(0), 4096),
                    (NodeId(1), -4106),
                    (NodeId(2), 7),
                ]),
            }
        );
        assert_eq!(delta.get(NodeId(2)), Some(7));
        assert!(delta.has_nonzero_delta());
        assert!(!zero_delta.has_nonzero_delta());
    }
}
