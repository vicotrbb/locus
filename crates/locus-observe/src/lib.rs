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

/// How an address matched a parsed `numa_maps` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaMapsAddressMatchKind {
    /// Address equals the mapping start address.
    ExactStart,
    /// Address falls inside the range inferred from adjacent entry starts.
    ContainingRange,
}

impl NumaMapsAddressMatchKind {
    /// Returns a stable machine-readable match kind string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactStart => "exact_start",
            Self::ContainingRange => "containing_range",
        }
    }
}

impl fmt::Display for NumaMapsAddressMatchKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Matched `numa_maps` entry for an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumaMapsAddressMatch<'a> {
    /// Match kind.
    pub kind: NumaMapsAddressMatchKind,
    /// Matched entry.
    pub entry: &'a NumaMapsEntry,
}

/// One parsed mapping range from `/proc/<pid>/smaps`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmapsEntry {
    /// Mapping start address.
    pub start_address: u64,
    /// Exclusive mapping end address.
    pub end_address: u64,
    /// Kernel-reported page size in KiB, from `KernelPageSize`.
    pub kernel_page_kb: Option<u64>,
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

/// Fault counters parsed from `/proc/<pid>/stat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessFaultCounts {
    /// Minor faults for the process.
    pub minor_faults: u64,
    /// Minor faults waited for by child processes.
    pub child_minor_faults: u64,
    /// Major faults for the process.
    pub major_faults: u64,
    /// Major faults waited for by child processes.
    pub child_major_faults: u64,
}

impl ProcessFaultCounts {
    /// Computes signed fault deltas from `previous` to `self`.
    #[must_use]
    pub fn delta_since(self, previous: Self) -> ProcessFaultDelta {
        ProcessFaultDelta {
            minor_faults_delta: i128::from(self.minor_faults) - i128::from(previous.minor_faults),
            child_minor_faults_delta: i128::from(self.child_minor_faults)
                - i128::from(previous.child_minor_faults),
            major_faults_delta: i128::from(self.major_faults) - i128::from(previous.major_faults),
            child_major_faults_delta: i128::from(self.child_major_faults)
                - i128::from(previous.child_major_faults),
        }
    }
}

/// Signed delta between two process fault counter snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessFaultDelta {
    /// Signed process minor-fault delta.
    pub minor_faults_delta: i128,
    /// Signed child minor-fault delta.
    pub child_minor_faults_delta: i128,
    /// Signed process major-fault delta.
    pub major_faults_delta: i128,
    /// Signed child major-fault delta.
    pub child_major_faults_delta: i128,
}

impl ProcessFaultDelta {
    /// Returns true when any fault counter changed.
    #[must_use]
    pub fn has_nonzero_delta(self) -> bool {
        self.minor_faults_delta != 0
            || self.child_minor_faults_delta != 0
            || self.major_faults_delta != 0
            || self.child_major_faults_delta != 0
    }
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

/// Final proof status for a mapping placement attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPlacementProofStatus {
    /// Primary evidence proves the mapping is fully on the expected node.
    Verified,
    /// Primary evidence is present but does not prove placement.
    Unverified,
    /// Primary evidence could not be collected.
    Unavailable,
}

impl NumaPlacementProofStatus {
    /// Returns a stable machine-readable proof status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Unverified => "unverified",
            Self::Unavailable => "unavailable",
        }
    }

    /// Parses a stable machine-readable proof status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "verified" => Some(Self::Verified),
            "unverified" => Some(Self::Unverified),
            "unavailable" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

impl fmt::Display for NumaPlacementProofStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for a placement proof status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPlacementProofReason {
    /// All primary proof conditions were satisfied.
    Verified,
    /// Linux memory policy application did not succeed.
    PolicyNotApplied,
    /// The target mapping was not found in `numa_maps`.
    MappingMissing,
    /// The matched mapping reported no materialized pages.
    NoPagesReported,
    /// The matched mapping reported both expected-node and other-node pages.
    PartialPagesOnExpectedNode,
    /// The matched mapping reported pages only on other nodes.
    NoPagesOnExpectedNode,
    /// The `numa_maps` file was unavailable.
    NumaMapsUnavailable,
}

impl NumaPlacementProofReason {
    /// Returns a stable machine-readable proof reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::PolicyNotApplied => "policy_not_applied",
            Self::MappingMissing => "mapping_missing",
            Self::NoPagesReported => "no_pages_reported",
            Self::PartialPagesOnExpectedNode => "partial_pages_on_expected_node",
            Self::NoPagesOnExpectedNode => "no_pages_on_expected_node",
            Self::NumaMapsUnavailable => "numa_maps_unavailable",
        }
    }

    /// Parses a stable machine-readable proof reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "verified" => Some(Self::Verified),
            "policy_not_applied" => Some(Self::PolicyNotApplied),
            "mapping_missing" => Some(Self::MappingMissing),
            "no_pages_reported" => Some(Self::NoPagesReported),
            "partial_pages_on_expected_node" => Some(Self::PartialPagesOnExpectedNode),
            "no_pages_on_expected_node" => Some(Self::NoPagesOnExpectedNode),
            "numa_maps_unavailable" => Some(Self::NumaMapsUnavailable),
            _ => None,
        }
    }
}

impl fmt::Display for NumaPlacementProofReason {
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

/// Final primary proof verdict for one placement attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumaPlacementProof {
    /// Final proof status.
    pub status: NumaPlacementProofStatus,
    /// Reason for the status.
    pub reason: NumaPlacementProofReason,
}

impl fmt::Display for NumaPlacementProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "placement_proof={} reason={}", self.status, self.reason)
    }
}

/// Error returned when parsing a probe placement proof line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumaPlacementProofLineParseError {
    /// The line does not contain a `placement_proof=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `placement_proof=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the placement proof line schema.
    InvalidToken(String),
    /// The proof status token is not recognized.
    UnknownStatus(String),
    /// The proof reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentProof {
        /// Parsed proof status.
        status: NumaPlacementProofStatus,
        /// Parsed proof reason.
        reason: NumaPlacementProofReason,
    },
}

impl fmt::Display for NumaPlacementProofLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing placement_proof token"),
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => f.write_str("duplicate placement_proof token"),
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => write!(f, "invalid placement proof token: {token}"),
            Self::UnknownStatus(status) => write!(f, "unknown placement proof status: {status}"),
            Self::UnknownReason(reason) => write!(f, "unknown placement proof reason: {reason}"),
            Self::InconsistentProof { status, reason } => {
                write!(f, "inconsistent placement proof: {status} {reason}")
            }
        }
    }
}

impl std::error::Error for NumaPlacementProofLineParseError {}

/// Error returned when extracting a placement proof from multiline probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumaPlacementProofOutputParseError {
    /// The output does not contain a `placement_proof=` line.
    MissingProofLine,
    /// The output contains more than one `placement_proof=` line.
    DuplicateProofLine,
    /// The discovered proof line is malformed.
    Line(NumaPlacementProofLineParseError),
}

impl fmt::Display for NumaPlacementProofOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingProofLine => f.write_str("missing placement_proof line"),
            Self::DuplicateProofLine => f.write_str("duplicate placement_proof line"),
            Self::Line(source) => write!(f, "invalid placement_proof line: {source}"),
        }
    }
}

impl std::error::Error for NumaPlacementProofOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingProofLine | Self::DuplicateProofLine => None,
        }
    }
}

/// Final readiness status for a NUMA placement validation environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPlacementValidationReadinessStatus {
    /// Required locality evidence sources are available.
    Ready,
    /// One or more required locality evidence sources are unavailable.
    NotReady,
}

impl NumaPlacementValidationReadinessStatus {
    /// Returns a stable machine-readable readiness status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
        }
    }

    /// Parses a stable machine-readable readiness status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "not_ready" => Some(Self::NotReady),
            _ => None,
        }
    }
}

impl fmt::Display for NumaPlacementValidationReadinessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for a NUMA placement validation readiness status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPlacementValidationReadinessReason {
    /// Required locality evidence sources are available.
    Ready,
    /// `/proc/self/numa_maps` is unavailable.
    NumaMapsUnavailable,
    /// Current cgroup v2 `memory.numa_stat` is unavailable.
    CgroupNumaStatUnavailable,
    /// Node `numastat` counters are unavailable.
    NodeNumastatUnavailable,
}

impl NumaPlacementValidationReadinessReason {
    /// Returns a stable machine-readable readiness reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NumaMapsUnavailable => "numa_maps_unavailable",
            Self::CgroupNumaStatUnavailable => "cgroup_numa_stat_unavailable",
            Self::NodeNumastatUnavailable => "node_numastat_unavailable",
        }
    }

    /// Parses a stable machine-readable readiness reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "numa_maps_unavailable" => Some(Self::NumaMapsUnavailable),
            "cgroup_numa_stat_unavailable" => Some(Self::CgroupNumaStatUnavailable),
            "node_numastat_unavailable" => Some(Self::NodeNumastatUnavailable),
            _ => None,
        }
    }
}

impl fmt::Display for NumaPlacementValidationReadinessReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Readiness verdict for collecting placement validation evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumaPlacementValidationReadiness {
    /// Final readiness status.
    pub status: NumaPlacementValidationReadinessStatus,
    /// Reason for the status.
    pub reason: NumaPlacementValidationReadinessReason,
}

impl fmt::Display for NumaPlacementValidationReadiness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "placement_validation_readiness={} reason={}",
            self.status, self.reason
        )
    }
}

/// Error returned when parsing a placement validation readiness line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumaPlacementReadinessLineParseError {
    /// The line does not contain a `placement_validation_readiness=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `placement_validation_readiness=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the readiness line schema.
    InvalidToken(String),
    /// The readiness status token is not recognized.
    UnknownStatus(String),
    /// The readiness reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentReadiness {
        /// Parsed readiness status.
        status: NumaPlacementValidationReadinessStatus,
        /// Parsed readiness reason.
        reason: NumaPlacementValidationReadinessReason,
    },
}

impl fmt::Display for NumaPlacementReadinessLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing placement_validation_readiness token"),
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => f.write_str("duplicate placement_validation_readiness token"),
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => write!(f, "invalid placement readiness token: {token}"),
            Self::UnknownStatus(status) => {
                write!(f, "unknown placement readiness status: {status}")
            }
            Self::UnknownReason(reason) => {
                write!(f, "unknown placement readiness reason: {reason}")
            }
            Self::InconsistentReadiness { status, reason } => {
                write!(f, "inconsistent placement readiness: {status} {reason}")
            }
        }
    }
}

impl std::error::Error for NumaPlacementReadinessLineParseError {}

/// Error returned when extracting placement validation readiness from multiline output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumaPlacementReadinessOutputParseError {
    /// The output does not contain a `placement_validation_readiness=` line.
    MissingReadinessLine,
    /// The output contains more than one `placement_validation_readiness=` line.
    DuplicateReadinessLine,
    /// The discovered readiness line is malformed.
    Line(NumaPlacementReadinessLineParseError),
}

impl fmt::Display for NumaPlacementReadinessOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingReadinessLine => {
                f.write_str("missing placement_validation_readiness line")
            }
            Self::DuplicateReadinessLine => {
                f.write_str("duplicate placement_validation_readiness line")
            }
            Self::Line(source) => {
                write!(f, "invalid placement_validation_readiness line: {source}")
            }
        }
    }
}

impl std::error::Error for NumaPlacementReadinessOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingReadinessLine | Self::DuplicateReadinessLine => None,
        }
    }
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

    /// Returns true only when every reported page is on the expected node.
    #[must_use]
    pub fn is_fully_on_expected_node(&self) -> bool {
        self.status == NumaPlacementStatus::AllPagesOnExpectedNode
    }

    /// Returns the number of pages reported on nodes other than the expected node.
    #[must_use]
    pub fn other_pages(&self) -> u64 {
        self.other_node_pages
            .values()
            .copied()
            .fold(0_u64, u64::saturating_add)
    }
}

impl NumaPlacementProof {
    /// Builds a proof verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: NumaPlacementProofStatus,
        reason: NumaPlacementProofReason,
    ) -> Result<Self, NumaPlacementProofLineParseError> {
        let proof = Self { status, reason };
        if proof.is_consistent() {
            Ok(proof)
        } else {
            Err(NumaPlacementProofLineParseError::InconsistentProof { status, reason })
        }
    }

    /// Builds a placement proof verdict from policy and `numa_maps` evidence.
    #[must_use]
    pub fn from_evidence(policy_applied: bool, evidence: Option<&NumaPlacementEvidence>) -> Self {
        if !policy_applied {
            return Self {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::PolicyNotApplied,
            };
        }

        let Some(evidence) = evidence else {
            return Self {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::MappingMissing,
            };
        };

        match evidence.status {
            NumaPlacementStatus::AllPagesOnExpectedNode => Self {
                status: NumaPlacementProofStatus::Verified,
                reason: NumaPlacementProofReason::Verified,
            },
            NumaPlacementStatus::NoPagesReported => Self {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::NoPagesReported,
            },
            NumaPlacementStatus::PartialPagesOnExpectedNode => Self {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::PartialPagesOnExpectedNode,
            },
            NumaPlacementStatus::NoPagesOnExpectedNode => Self {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::NoPagesOnExpectedNode,
            },
        }
    }

    /// Returns true only when primary evidence proves placement.
    #[must_use]
    pub fn is_verified(self) -> bool {
        self.status == NumaPlacementProofStatus::Verified
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                NumaPlacementProofStatus::Verified,
                NumaPlacementProofReason::Verified
            ) | (
                NumaPlacementProofStatus::Unverified,
                NumaPlacementProofReason::PolicyNotApplied
                    | NumaPlacementProofReason::MappingMissing
                    | NumaPlacementProofReason::NoPagesReported
                    | NumaPlacementProofReason::PartialPagesOnExpectedNode
                    | NumaPlacementProofReason::NoPagesOnExpectedNode
            ) | (
                NumaPlacementProofStatus::Unavailable,
                NumaPlacementProofReason::NumaMapsUnavailable
            )
        )
    }

    /// Builds a placement proof verdict for unavailable primary evidence.
    #[must_use]
    pub fn unavailable(reason: NumaPlacementProofReason) -> Self {
        Self {
            status: NumaPlacementProofStatus::Unavailable,
            reason,
        }
    }
}

/// Parses a mapped scratch bind probe placement proof line.
///
/// The expected format is `placement_proof=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_numa_placement_proof_line(
    line: &str,
) -> Result<NumaPlacementProof, NumaPlacementProofLineParseError> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(NumaPlacementProofLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "placement_proof" => {
                if status_token.replace(value).is_some() {
                    return Err(NumaPlacementProofLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(NumaPlacementProofLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(NumaPlacementProofLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            }
        }
    }

    let status_token = status_token.ok_or(NumaPlacementProofLineParseError::MissingStatus)?;
    let reason_token = reason_token.ok_or(NumaPlacementProofLineParseError::MissingReason)?;

    let status = NumaPlacementProofStatus::from_str_token(status_token)
        .ok_or_else(|| NumaPlacementProofLineParseError::UnknownStatus(status_token.to_owned()))?;
    let reason = NumaPlacementProofReason::from_str_token(reason_token)
        .ok_or_else(|| NumaPlacementProofLineParseError::UnknownReason(reason_token.to_owned()))?;

    NumaPlacementProof::from_parts(status, reason)
}

/// Extracts the final placement proof verdict from multiline probe output.
///
/// # Errors
///
/// Returns an error when the output has no placement proof line, has more than
/// one placement proof line, or contains a malformed placement proof line.
pub fn parse_numa_placement_proof_output(
    output: &str,
) -> Result<NumaPlacementProof, NumaPlacementProofOutputParseError> {
    let mut proof = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("placement_proof=") {
            continue;
        }

        if proof.is_some() {
            return Err(NumaPlacementProofOutputParseError::DuplicateProofLine);
        }

        proof = Some(
            parse_numa_placement_proof_line(line)
                .map_err(NumaPlacementProofOutputParseError::Line)?,
        );
    }

    proof.ok_or(NumaPlacementProofOutputParseError::MissingProofLine)
}

impl NumaPlacementValidationReadiness {
    /// Builds a readiness verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: NumaPlacementValidationReadinessStatus,
        reason: NumaPlacementValidationReadinessReason,
    ) -> Result<Self, NumaPlacementReadinessLineParseError> {
        let readiness = Self { status, reason };
        if readiness.is_consistent() {
            Ok(readiness)
        } else {
            Err(NumaPlacementReadinessLineParseError::InconsistentReadiness { status, reason })
        }
    }

    /// Builds a readiness verdict from locality evidence availability.
    #[must_use]
    pub fn from_sources(
        numa_maps_available: bool,
        cgroup_numa_stat_available: bool,
        node_numastat_available: bool,
    ) -> Self {
        if !numa_maps_available {
            return Self {
                status: NumaPlacementValidationReadinessStatus::NotReady,
                reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
            };
        }

        if !cgroup_numa_stat_available {
            return Self {
                status: NumaPlacementValidationReadinessStatus::NotReady,
                reason: NumaPlacementValidationReadinessReason::CgroupNumaStatUnavailable,
            };
        }

        if !node_numastat_available {
            return Self {
                status: NumaPlacementValidationReadinessStatus::NotReady,
                reason: NumaPlacementValidationReadinessReason::NodeNumastatUnavailable,
            };
        }

        Self {
            status: NumaPlacementValidationReadinessStatus::Ready,
            reason: NumaPlacementValidationReadinessReason::Ready,
        }
    }

    /// Returns true only when all required locality evidence sources are available.
    #[must_use]
    pub fn is_ready(self) -> bool {
        self.status == NumaPlacementValidationReadinessStatus::Ready
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                NumaPlacementValidationReadinessStatus::Ready,
                NumaPlacementValidationReadinessReason::Ready
            ) | (
                NumaPlacementValidationReadinessStatus::NotReady,
                NumaPlacementValidationReadinessReason::NumaMapsUnavailable
                    | NumaPlacementValidationReadinessReason::CgroupNumaStatUnavailable
                    | NumaPlacementValidationReadinessReason::NodeNumastatUnavailable
            )
        )
    }
}

/// Parses a locality environment placement validation readiness line.
///
/// The expected format is `placement_validation_readiness=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_numa_placement_readiness_line(
    line: &str,
) -> Result<NumaPlacementValidationReadiness, NumaPlacementReadinessLineParseError> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(NumaPlacementReadinessLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "placement_validation_readiness" => {
                if status_token.replace(value).is_some() {
                    return Err(NumaPlacementReadinessLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(NumaPlacementReadinessLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(NumaPlacementReadinessLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            }
        }
    }

    let status_token = status_token.ok_or(NumaPlacementReadinessLineParseError::MissingStatus)?;
    let reason_token = reason_token.ok_or(NumaPlacementReadinessLineParseError::MissingReason)?;

    let status =
        NumaPlacementValidationReadinessStatus::from_str_token(status_token).ok_or_else(|| {
            NumaPlacementReadinessLineParseError::UnknownStatus(status_token.to_owned())
        })?;
    let reason =
        NumaPlacementValidationReadinessReason::from_str_token(reason_token).ok_or_else(|| {
            NumaPlacementReadinessLineParseError::UnknownReason(reason_token.to_owned())
        })?;

    NumaPlacementValidationReadiness::from_parts(status, reason)
}

/// Extracts placement validation readiness from multiline probe output.
///
/// # Errors
///
/// Returns an error when the output has no placement validation readiness line,
/// has more than one placement validation readiness line, or contains a
/// malformed placement validation readiness line.
pub fn parse_numa_placement_readiness_output(
    output: &str,
) -> Result<NumaPlacementValidationReadiness, NumaPlacementReadinessOutputParseError> {
    let mut readiness = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("placement_validation_readiness=") {
            continue;
        }

        if readiness.is_some() {
            return Err(NumaPlacementReadinessOutputParseError::DuplicateReadinessLine);
        }

        readiness = Some(
            parse_numa_placement_readiness_line(line)
                .map_err(NumaPlacementReadinessOutputParseError::Line)?,
        );
    }

    readiness.ok_or(NumaPlacementReadinessOutputParseError::MissingReadinessLine)
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

/// Reads and parses a `smaps` file from an explicit path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsing fails.
pub fn read_smaps(path: impl AsRef<Path>) -> Result<Vec<SmapsEntry>, ObserveReadError> {
    let path = path.as_ref();
    let input = read_to_string(path)?;
    parse_smaps(&input).map_err(ObserveReadError::Parse)
}

/// Reads and parses `/proc/self/smaps`.
///
/// # Errors
///
/// Returns an error when `/proc/self/smaps` cannot be read or parsing fails.
/// Non-Linux hosts normally return a read error.
pub fn read_self_smaps() -> Result<Vec<SmapsEntry>, ObserveReadError> {
    read_smaps("/proc/self/smaps")
}

/// Reads and parses process fault counters from an explicit proc stat path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsing fails.
pub fn read_process_fault_counts(
    path: impl AsRef<Path>,
) -> Result<ProcessFaultCounts, ObserveReadError> {
    let path = path.as_ref();
    let input = read_to_string(path)?;
    parse_process_stat_fault_counts(&input).map_err(ObserveReadError::Parse)
}

/// Reads and parses `/proc/self/stat` process fault counters.
///
/// # Errors
///
/// Returns an error when `/proc/self/stat` cannot be read or parsing fails.
/// Non-Linux hosts normally return a read error.
pub fn read_self_process_fault_counts() -> Result<ProcessFaultCounts, ObserveReadError> {
    read_process_fault_counts("/proc/self/stat")
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

/// Finds the best `numa_maps` entry match for an address.
///
/// Exact start matches are preferred over containing range matches.
#[must_use]
pub fn numa_maps_entry_for_address(
    entries: &[NumaMapsEntry],
    address: usize,
) -> Option<NumaMapsAddressMatch<'_>> {
    if let Some(entry) = numa_maps_entry_by_start_address(entries, address) {
        return Some(NumaMapsAddressMatch {
            kind: NumaMapsAddressMatchKind::ExactStart,
            entry,
        });
    }

    numa_maps_entry_containing_address(entries, address).map(|entry| NumaMapsAddressMatch {
        kind: NumaMapsAddressMatchKind::ContainingRange,
        entry,
    })
}

/// Finds the `smaps` entry containing an address.
#[must_use]
pub fn smaps_entry_for_address(entries: &[SmapsEntry], address: usize) -> Option<&SmapsEntry> {
    let address = u64::try_from(address).ok()?;

    entries
        .iter()
        .find(|entry| entry.start_address <= address && address < entry.end_address)
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

/// Reads and summarizes cgroup v2 `memory.numa_stat` from an explicit path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed.
pub fn read_cgroup_numa_summary(
    path: impl AsRef<Path>,
) -> Result<CgroupNumaSummary, ObserveReadError> {
    read_cgroup_numa_stat(path).map(|entries| CgroupNumaSummary::from_entries(&entries))
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

/// Parses mapping ranges and kernel page sizes from `/proc/<pid>/smaps`.
///
/// # Errors
///
/// Returns an error when a mapping header has an invalid address range or a
/// `KernelPageSize` field has a malformed numeric value.
pub fn parse_smaps(input: &str) -> Result<Vec<SmapsEntry>, ObserveParseError> {
    let mut entries = Vec::new();
    let mut current = None;

    for (index, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(range) = line
            .split_whitespace()
            .next()
            .filter(|token| token.contains('-'))
        {
            if let Some(entry) = current.replace(parse_smaps_header(range).map_err(|source| {
                ObserveParseError::Line {
                    line: index + 1,
                    source: Box::new(source),
                }
            })?) {
                entries.push(entry);
            }
            continue;
        }

        let Some(entry) = current.as_mut() else {
            return Err(ObserveParseError::Line {
                line: index + 1,
                source: Box::new(ObserveParseError::InvalidToken(line.to_owned())),
            });
        };

        if let Some(value) = line.strip_prefix("KernelPageSize:") {
            let page_kb =
                parse_smaps_kb_field(line, value).map_err(|source| ObserveParseError::Line {
                    line: index + 1,
                    source: Box::new(source),
                })?;
            entry.kernel_page_kb = Some(page_kb);
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    Ok(entries)
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

/// Parses fault counters from `/proc/<pid>/stat` content.
///
/// # Errors
///
/// Returns an error when the stat content is missing required fields or fault
/// fields are malformed.
pub fn parse_process_stat_fault_counts(
    input: &str,
) -> Result<ProcessFaultCounts, ObserveParseError> {
    let tail = process_stat_after_command(input)?;
    let mut tokens = tail.split_whitespace();

    let _state = tokens
        .next()
        .ok_or(ObserveParseError::MissingProcStatField("state"))?;

    for field in ["ppid", "pgrp", "session", "tty_nr", "tpgid", "flags"] {
        tokens
            .next()
            .ok_or(ObserveParseError::MissingProcStatField(field))?;
    }

    let minor_faults = parse_process_stat_u64(
        tokens
            .next()
            .ok_or(ObserveParseError::MissingProcStatField("minflt"))?,
        "minflt",
    )?;
    let child_minor_faults = parse_process_stat_u64(
        tokens
            .next()
            .ok_or(ObserveParseError::MissingProcStatField("cminflt"))?,
        "cminflt",
    )?;
    let major_faults = parse_process_stat_u64(
        tokens
            .next()
            .ok_or(ObserveParseError::MissingProcStatField("majflt"))?,
        "majflt",
    )?;
    let child_major_faults = parse_process_stat_u64(
        tokens
            .next()
            .ok_or(ObserveParseError::MissingProcStatField("cmajflt"))?,
        "cmajflt",
    )?;

    Ok(ProcessFaultCounts {
        minor_faults,
        child_minor_faults,
        major_faults,
        child_major_faults,
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
    /// A `/proc/<pid>/stat` field was missing.
    MissingProcStatField(&'static str),
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
            Self::MissingProcStatField(field) => {
                write!(f, "missing proc stat field: {field}")
            }
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

fn parse_smaps_header(range: &str) -> Result<SmapsEntry, ObserveParseError> {
    let (start, end) = range
        .split_once('-')
        .ok_or_else(|| ObserveParseError::InvalidAddress(range.to_owned()))?;
    if start.is_empty() || end.is_empty() {
        return Err(ObserveParseError::InvalidAddress(range.to_owned()));
    }

    let start_address = u64::from_str_radix(start, 16)
        .map_err(|_| ObserveParseError::InvalidAddress(range.to_owned()))?;
    let end_address = u64::from_str_radix(end, 16)
        .map_err(|_| ObserveParseError::InvalidAddress(range.to_owned()))?;

    if start_address >= end_address {
        return Err(ObserveParseError::InvalidAddress(range.to_owned()));
    }

    Ok(SmapsEntry {
        start_address,
        end_address,
        kernel_page_kb: None,
    })
}

fn parse_smaps_kb_field(line: &str, value: &str) -> Result<u64, ObserveParseError> {
    let mut tokens = value.split_whitespace();
    let number = tokens
        .next()
        .ok_or_else(|| ObserveParseError::InvalidToken(line.to_owned()))?;
    let unit = tokens
        .next()
        .ok_or_else(|| ObserveParseError::InvalidToken(line.to_owned()))?;

    if unit != "kB" || tokens.next().is_some() {
        return Err(ObserveParseError::InvalidToken(line.to_owned()));
    }

    parse_u64(number, line)
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

fn process_stat_after_command(input: &str) -> Result<&str, ObserveParseError> {
    let close_command = input
        .rfind(')')
        .ok_or_else(|| ObserveParseError::InvalidToken(input.trim().to_owned()))?;
    Ok(&input[close_command + 1..])
}

fn parse_process_stat_u64(value: &str, field: &str) -> Result<u64, ObserveParseError> {
    parse_u64(value, field)
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
        numa_maps_entry_for_address, parse_cgroup_numa_stat, parse_node_numastat, parse_numa_maps,
        parse_numa_maps_line, parse_numa_placement_proof_line, parse_numa_placement_proof_output,
        parse_numa_placement_readiness_line, parse_numa_placement_readiness_output,
        parse_process_stat_fault_counts, parse_smaps, read_cgroup_numa_stat,
        read_cgroup_numa_summary, read_node_numastat, read_node_numastat_system_snapshot,
        read_numa_maps, read_process_fault_counts, resolve_cgroup_v2_memory_numa_stat_path,
        smaps_entry_for_address, CgroupNumaDelta, CgroupNumaSummary, CgroupPathError,
        NodeNumastatSnapshot, NodeNumastatSystemSnapshot, NumaMapsAddressMatchKind,
        NumaMapsSummary, NumaPlacementEvidence, NumaPlacementProof,
        NumaPlacementProofLineParseError, NumaPlacementProofOutputParseError,
        NumaPlacementProofReason, NumaPlacementProofStatus, NumaPlacementReadinessLineParseError,
        NumaPlacementReadinessOutputParseError, NumaPlacementStatus,
        NumaPlacementValidationReadiness, NumaPlacementValidationReadinessReason,
        NumaPlacementValidationReadinessStatus, ObserveParseError, ProcessFaultCounts,
        ProcessFaultDelta, SmapsEntry,
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
    fn finds_best_numa_maps_address_match() {
        let entries = parse_numa_maps(
            "1000 default anon=1 N0=1\n\
             2000 bind:0 anon=2 N0=2\n\
             4000 default anon=1 N1=1\n",
        )
        .expect("valid numa maps");

        let exact = numa_maps_entry_for_address(&entries, 0x2000).expect("exact match");
        let containing = numa_maps_entry_for_address(&entries, 0x2fff).expect("containing match");

        assert_eq!(exact.kind, NumaMapsAddressMatchKind::ExactStart);
        assert_eq!(exact.kind.to_string(), "exact_start");
        assert_eq!(exact.entry.policy, "bind:0");
        assert_eq!(containing.kind, NumaMapsAddressMatchKind::ContainingRange);
        assert_eq!(containing.entry.policy, "bind:0");
        assert!(numa_maps_entry_for_address(&entries, 0x0fff).is_none());
    }

    #[test]
    fn parses_smaps_ranges_and_kernel_page_sizes() {
        let entries = parse_smaps(
            "1000-2000 rw-p 00000000 00:00 0\n\
             Size:                  4 kB\n\
             KernelPageSize:        4 kB\n\
             MMUPageSize:           4 kB\n\
             VmFlags: rd wr mr mw me ac sd\n\
             \n\
             2000-4000 rw-p 00000000 00:00 0\n\
             Size:               8192 kB\n\
             KernelPageSize:     2048 kB\n",
        )
        .expect("valid smaps");

        assert_eq!(
            entries,
            vec![
                SmapsEntry {
                    start_address: 0x1000,
                    end_address: 0x2000,
                    kernel_page_kb: Some(4),
                },
                SmapsEntry {
                    start_address: 0x2000,
                    end_address: 0x4000,
                    kernel_page_kb: Some(2048),
                },
            ]
        );
        assert_eq!(
            smaps_entry_for_address(&entries, 0x2000).map(|entry| entry.kernel_page_kb),
            Some(Some(2048))
        );
        assert!(smaps_entry_for_address(&entries, 0x4000).is_none());
    }

    #[test]
    fn parses_smaps_entry_without_kernel_page_size() {
        let entries =
            parse_smaps("1000-2000 rw-p 00000000 00:00 0\nSize: 4 kB\n").expect("valid smaps");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kernel_page_kb, None);
    }

    #[test]
    fn rejects_malformed_smaps_header() {
        let error = parse_smaps("2000-1000 rw-p 00000000 00:00 0\n").expect_err("invalid range");

        assert!(matches!(
            error,
            ObserveParseError::Line { source, .. }
                if matches!(*source, ObserveParseError::InvalidAddress(_))
        ));
    }

    #[test]
    fn rejects_malformed_smaps_kernel_page_size() {
        let error = parse_smaps(
            "1000-2000 rw-p 00000000 00:00 0\n\
             KernelPageSize: 4 bytes\n",
        )
        .expect_err("invalid kernel page size");

        assert!(matches!(
            error,
            ObserveParseError::Line { source, .. }
                if matches!(*source, ObserveParseError::InvalidToken(_))
        ));
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
    fn parses_process_stat_fault_counts_with_spaced_command_name() {
        let counts = parse_process_stat_fault_counts(
            "123 (locus bench (probe)) R 1 2 3 4 5 6 100 7 8 9 0 0 0 0\n",
        )
        .expect("valid process stat");

        assert_eq!(
            counts,
            ProcessFaultCounts {
                minor_faults: 100,
                child_minor_faults: 7,
                major_faults: 8,
                child_major_faults: 9,
            }
        );
    }

    #[test]
    fn rejects_invalid_process_stat_fault_counts() {
        assert_eq!(
            parse_process_stat_fault_counts("123 locus R 1 2 3 4 5 6 100 7 8 9\n")
                .expect_err("missing command delimiter"),
            ObserveParseError::InvalidToken("123 locus R 1 2 3 4 5 6 100 7 8 9".to_owned())
        );
        assert_eq!(
            parse_process_stat_fault_counts("123 (locus) R 1 2 3 4 5 6 100 7 8\n")
                .expect_err("missing cmajflt"),
            ObserveParseError::MissingProcStatField("cmajflt")
        );
        assert_eq!(
            parse_process_stat_fault_counts("123 (locus) R 1 2 3 4 5 6 bad 7 8 9\n")
                .expect_err("bad minflt"),
            ObserveParseError::InvalidNumber {
                token: "minflt".to_owned(),
                value: "bad".to_owned(),
            }
        );
    }

    #[test]
    fn computes_process_fault_count_delta() {
        let before = ProcessFaultCounts {
            minor_faults: 10,
            child_minor_faults: 5,
            major_faults: 2,
            child_major_faults: 1,
        };
        let after = ProcessFaultCounts {
            minor_faults: 15,
            child_minor_faults: 3,
            major_faults: 2,
            child_major_faults: 4,
        };

        let delta = after.delta_since(before);
        let zero_delta = before.delta_since(before);

        assert_eq!(
            delta,
            ProcessFaultDelta {
                minor_faults_delta: 5,
                child_minor_faults_delta: -2,
                major_faults_delta: 0,
                child_major_faults_delta: 3,
            }
        );
        assert!(delta.has_nonzero_delta());
        assert!(!zero_delta.has_nonzero_delta());
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
        let process_stat = temp.path().join("stat");

        fs::write(
            &numa_maps,
            "7f6c00000000 default anon=4 dirty=4 N0=4 kernelpagesize_kB=4\n",
        )
        .expect("write numa_maps");
        fs::write(&cgroup_stat, "anon N0=4096 N1=8192\n").expect("write cgroup stat");
        fs::write(&node_stat, "numa_hit 10\nother_node 1\n").expect("write node stat");
        fs::write(
            &process_stat,
            "123 (locus bench (probe)) R 1 2 3 4 5 6 100 7 8 9 0 0 0 0\n",
        )
        .expect("write process stat");

        assert_eq!(read_numa_maps(&numa_maps).expect("read numa maps").len(), 1);
        assert_eq!(
            read_cgroup_numa_stat(&cgroup_stat)
                .expect("read cgroup stat")
                .len(),
            1
        );
        assert_eq!(
            read_cgroup_numa_summary(&cgroup_stat)
                .expect("read cgroup summary")
                .total_bytes,
            12_288
        );
        assert_eq!(
            read_node_numastat(&node_stat)
                .expect("read node stat")
                .len(),
            2
        );
        assert_eq!(
            read_process_fault_counts(&process_stat).expect("read process stat"),
            ProcessFaultCounts {
                minor_faults: 100,
                child_minor_faults: 7,
                major_faults: 8,
                child_major_faults: 9,
            }
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
        assert!(all_local.is_fully_on_expected_node());
        assert_eq!(all_local.other_pages(), 0);
        assert_eq!(
            partial.status,
            NumaPlacementStatus::PartialPagesOnExpectedNode
        );
        assert_eq!(partial.other_node_pages.get(&NodeId(1)), Some(&3));
        assert!(!partial.is_fully_on_expected_node());
        assert_eq!(partial.other_pages(), 3);
        assert_eq!(remote.status, NumaPlacementStatus::NoPagesOnExpectedNode);
        assert_eq!(no_pages.status, NumaPlacementStatus::NoPagesReported);
        assert_eq!(
            NumaPlacementStatus::AllPagesOnExpectedNode.to_string(),
            "all_pages_on_expected_node"
        );
    }

    #[test]
    fn reports_numa_placement_proof_verdicts() {
        let entries = parse_numa_maps(
            "1000 bind:0 anon=4 N0=4\n\
             2000 bind:0 anon=4 N0=2 N1=2\n\
             3000 bind:0 anon=4 N1=4\n\
             4000 bind:0 anon=0\n",
        )
        .expect("valid numa maps");
        let verified = NumaPlacementEvidence::from_entry(&entries[0], NodeId(0));
        let partial = NumaPlacementEvidence::from_entry(&entries[1], NodeId(0));
        let remote = NumaPlacementEvidence::from_entry(&entries[2], NodeId(0));
        let no_pages = NumaPlacementEvidence::from_entry(&entries[3], NodeId(0));

        let proof = NumaPlacementProof::from_evidence(true, Some(&verified));
        assert_eq!(proof.status, NumaPlacementProofStatus::Verified);
        assert_eq!(proof.reason, NumaPlacementProofReason::Verified);
        assert_eq!(proof.status.to_string(), "verified");
        assert_eq!(proof.reason.to_string(), "verified");
        assert_eq!(
            proof.to_string(),
            "placement_proof=verified reason=verified"
        );
        assert!(proof.is_verified());

        assert_eq!(
            NumaPlacementProof::from_evidence(false, Some(&verified)),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::PolicyNotApplied,
            }
        );
        assert_eq!(
            NumaPlacementProof::from_evidence(true, None),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::MappingMissing,
            }
        );
        assert_eq!(
            NumaPlacementProof::from_evidence(true, Some(&partial)).reason,
            NumaPlacementProofReason::PartialPagesOnExpectedNode
        );
        assert_eq!(
            NumaPlacementProof::from_evidence(true, Some(&remote)).reason,
            NumaPlacementProofReason::NoPagesOnExpectedNode
        );
        assert_eq!(
            NumaPlacementProof::from_evidence(true, Some(&no_pages)).reason,
            NumaPlacementProofReason::NoPagesReported
        );

        let unavailable =
            NumaPlacementProof::unavailable(NumaPlacementProofReason::NumaMapsUnavailable);
        assert_eq!(unavailable.status, NumaPlacementProofStatus::Unavailable);
        assert_eq!(
            unavailable.reason,
            NumaPlacementProofReason::NumaMapsUnavailable
        );
        assert_eq!(unavailable.status.to_string(), "unavailable");
        assert_eq!(unavailable.reason.to_string(), "numa_maps_unavailable");
        assert!(!unavailable.is_verified());
        assert!(unavailable.is_consistent());
        assert!(!NumaPlacementProof {
            status: NumaPlacementProofStatus::Verified,
            reason: NumaPlacementProofReason::PolicyNotApplied,
        }
        .is_consistent());
    }

    #[test]
    fn parses_numa_placement_proof_lines() {
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=verified reason=verified")
                .expect("verified proof"),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Verified,
                reason: NumaPlacementProofReason::Verified,
            }
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=unverified reason=policy_not_applied")
                .expect("unverified proof"),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Unverified,
                reason: NumaPlacementProofReason::PolicyNotApplied,
            }
        );
        assert_eq!(
            parse_numa_placement_proof_line(
                "placement_proof=unavailable reason=numa_maps_unavailable"
            )
            .expect("unavailable proof"),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Unavailable,
                reason: NumaPlacementProofReason::NumaMapsUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_numa_placement_proof_lines() {
        assert_eq!(
            parse_numa_placement_proof_line("reason=verified").expect_err("missing status"),
            NumaPlacementProofLineParseError::MissingStatus
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=verified")
                .expect_err("missing reason"),
            NumaPlacementProofLineParseError::MissingReason
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=maybe reason=verified")
                .expect_err("unknown status"),
            NumaPlacementProofLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=verified reason=maybe")
                .expect_err("unknown reason"),
            NumaPlacementProofLineParseError::UnknownReason("maybe".to_owned())
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=verified reason=verified extra=true")
                .expect_err("extra token"),
            NumaPlacementProofLineParseError::InvalidToken("extra=true".to_owned())
        );
        assert_eq!(
            parse_numa_placement_proof_line(
                "placement_proof=verified placement_proof=unverified reason=verified"
            )
            .expect_err("duplicate status"),
            NumaPlacementProofLineParseError::DuplicateStatus
        );
        assert_eq!(
            parse_numa_placement_proof_line(
                "placement_proof=verified reason=verified reason=policy_not_applied"
            )
            .expect_err("duplicate reason"),
            NumaPlacementProofLineParseError::DuplicateReason
        );
        assert_eq!(
            parse_numa_placement_proof_line("placement_proof=verified reason=policy_not_applied")
                .expect_err("inconsistent proof"),
            NumaPlacementProofLineParseError::InconsistentProof {
                status: NumaPlacementProofStatus::Verified,
                reason: NumaPlacementProofReason::PolicyNotApplied,
            }
        );
    }

    #[test]
    fn parses_numa_placement_proof_from_probe_output() {
        let output = "\
mapping_start=0xffffb9222000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
placement_proof=unavailable reason=numa_maps_unavailable
";

        assert_eq!(
            parse_numa_placement_proof_output(output).expect("proof"),
            NumaPlacementProof {
                status: NumaPlacementProofStatus::Unavailable,
                reason: NumaPlacementProofReason::NumaMapsUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_numa_placement_proof_output() {
        assert_eq!(
            parse_numa_placement_proof_output("numa_maps=unavailable\n")
                .expect_err("missing proof"),
            NumaPlacementProofOutputParseError::MissingProofLine
        );
        assert_eq!(
            parse_numa_placement_proof_output(
                "placement_proof=verified reason=verified\nplacement_proof=unverified reason=mapping_missing\n",
            )
            .expect_err("duplicate proof"),
            NumaPlacementProofOutputParseError::DuplicateProofLine
        );
        assert_eq!(
            parse_numa_placement_proof_output("placement_proof=maybe reason=verified\n")
                .expect_err("bad proof"),
            NumaPlacementProofOutputParseError::Line(
                NumaPlacementProofLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
        assert_eq!(
            parse_numa_placement_proof_output(
                "placement_proof=unavailable reason=mapping_missing\n"
            )
            .expect_err("inconsistent proof"),
            NumaPlacementProofOutputParseError::Line(
                NumaPlacementProofLineParseError::InconsistentProof {
                    status: NumaPlacementProofStatus::Unavailable,
                    reason: NumaPlacementProofReason::MappingMissing,
                }
            )
        );
    }

    #[test]
    fn reports_numa_placement_validation_readiness() {
        let ready = NumaPlacementValidationReadiness::from_sources(true, true, true);
        assert_eq!(ready.status, NumaPlacementValidationReadinessStatus::Ready);
        assert_eq!(ready.reason, NumaPlacementValidationReadinessReason::Ready);
        assert_eq!(ready.status.to_string(), "ready");
        assert_eq!(ready.reason.to_string(), "ready");
        assert_eq!(
            ready.to_string(),
            "placement_validation_readiness=ready reason=ready"
        );
        assert!(ready.is_ready());

        let missing_numa_maps = NumaPlacementValidationReadiness::from_sources(false, true, true);
        assert_eq!(
            missing_numa_maps.status,
            NumaPlacementValidationReadinessStatus::NotReady
        );
        assert_eq!(
            missing_numa_maps.reason,
            NumaPlacementValidationReadinessReason::NumaMapsUnavailable
        );
        assert_eq!(missing_numa_maps.status.to_string(), "not_ready");
        assert_eq!(
            missing_numa_maps.reason.to_string(),
            "numa_maps_unavailable"
        );
        assert!(!missing_numa_maps.is_ready());

        assert_eq!(
            NumaPlacementValidationReadiness::from_sources(true, false, true).reason,
            NumaPlacementValidationReadinessReason::CgroupNumaStatUnavailable
        );
        assert_eq!(
            NumaPlacementValidationReadiness::from_sources(true, true, false).reason,
            NumaPlacementValidationReadinessReason::NodeNumastatUnavailable
        );
        assert!(NumaPlacementValidationReadiness {
            status: NumaPlacementValidationReadinessStatus::NotReady,
            reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
        }
        .is_consistent());
        assert!(!NumaPlacementValidationReadiness {
            status: NumaPlacementValidationReadinessStatus::Ready,
            reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
        }
        .is_consistent());
    }

    #[test]
    fn parses_numa_placement_readiness_lines() {
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready reason=ready"
            )
            .expect("ready"),
            NumaPlacementValidationReadiness {
                status: NumaPlacementValidationReadinessStatus::Ready,
                reason: NumaPlacementValidationReadinessReason::Ready,
            }
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=not_ready reason=numa_maps_unavailable"
            )
            .expect("not ready"),
            NumaPlacementValidationReadiness {
                status: NumaPlacementValidationReadinessStatus::NotReady,
                reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_numa_placement_readiness_lines() {
        assert_eq!(
            parse_numa_placement_readiness_line("reason=ready").expect_err("missing status"),
            NumaPlacementReadinessLineParseError::MissingStatus
        );
        assert_eq!(
            parse_numa_placement_readiness_line("placement_validation_readiness=ready")
                .expect_err("missing reason"),
            NumaPlacementReadinessLineParseError::MissingReason
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=maybe reason=ready"
            )
            .expect_err("unknown status"),
            NumaPlacementReadinessLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready reason=maybe"
            )
            .expect_err("unknown reason"),
            NumaPlacementReadinessLineParseError::UnknownReason("maybe".to_owned())
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready reason=ready extra=true"
            )
            .expect_err("extra token"),
            NumaPlacementReadinessLineParseError::InvalidToken("extra=true".to_owned())
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready placement_validation_readiness=not_ready reason=ready"
            )
            .expect_err("duplicate status"),
            NumaPlacementReadinessLineParseError::DuplicateStatus
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready reason=ready reason=numa_maps_unavailable"
            )
            .expect_err("duplicate reason"),
            NumaPlacementReadinessLineParseError::DuplicateReason
        );
        assert_eq!(
            parse_numa_placement_readiness_line(
                "placement_validation_readiness=ready reason=numa_maps_unavailable"
            )
            .expect_err("inconsistent readiness"),
            NumaPlacementReadinessLineParseError::InconsistentReadiness {
                status: NumaPlacementValidationReadinessStatus::Ready,
                reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
            }
        );
    }

    #[test]
    fn parses_numa_placement_readiness_from_probe_output() {
        let output = "\
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
";

        assert_eq!(
            parse_numa_placement_readiness_output(output).expect("readiness"),
            NumaPlacementValidationReadiness {
                status: NumaPlacementValidationReadinessStatus::NotReady,
                reason: NumaPlacementValidationReadinessReason::NumaMapsUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_numa_placement_readiness_output() {
        assert_eq!(
            parse_numa_placement_readiness_output("numa_maps=unavailable\n")
                .expect_err("missing readiness"),
            NumaPlacementReadinessOutputParseError::MissingReadinessLine
        );
        assert_eq!(
            parse_numa_placement_readiness_output(
                "placement_validation_readiness=ready reason=ready\nplacement_validation_readiness=not_ready reason=numa_maps_unavailable\n",
            )
            .expect_err("duplicate readiness"),
            NumaPlacementReadinessOutputParseError::DuplicateReadinessLine
        );
        assert_eq!(
            parse_numa_placement_readiness_output(
                "placement_validation_readiness=maybe reason=ready\n"
            )
            .expect_err("bad readiness"),
            NumaPlacementReadinessOutputParseError::Line(
                NumaPlacementReadinessLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
        assert_eq!(
            parse_numa_placement_readiness_output(
                "placement_validation_readiness=not_ready reason=ready\n"
            )
            .expect_err("inconsistent readiness"),
            NumaPlacementReadinessOutputParseError::Line(
                NumaPlacementReadinessLineParseError::InconsistentReadiness {
                    status: NumaPlacementValidationReadinessStatus::NotReady,
                    reason: NumaPlacementValidationReadinessReason::Ready,
                }
            )
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
