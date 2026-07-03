use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use crate::remote_free_service_sample_compare::{
    compare_remote_free_service_telemetry_sample_outputs_with_timings,
    RemoteFreeServiceTelemetrySampleTimingCompareError,
    RemoteFreeServiceTelemetrySampleTimingCompareStatus, RemoteFreeServiceTelemetryTimingDelta,
};

/// Borrowed benchmark output used by the repeated-run stability summarizer.
#[derive(Debug, Clone, Copy)]
pub struct RemoteFreeServiceTelemetryTimingStabilityRun<'a> {
    /// Stable label printed in reports.
    pub label: &'a str,
    /// Saved Criterion output text.
    pub output: &'a str,
}

/// Role for one repeated-run timing stability manifest entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetryTimingStabilityManifestRole {
    /// Baseline output.
    Baseline,
    /// Candidate output compared with the baseline.
    Candidate,
}

impl RemoteFreeServiceTelemetryTimingStabilityManifestRole {
    /// Returns a stable machine-readable role string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityManifestRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One saved-output entry from a repeated-run timing stability manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingStabilityManifestEntry {
    /// Entry role.
    pub role: RemoteFreeServiceTelemetryTimingStabilityManifestRole,
    /// Stable run label.
    pub label: String,
    /// Saved output path.
    pub path: String,
}

/// Parsed repeated-run timing stability manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingStabilityManifest {
    /// Baseline saved output entry.
    pub baseline: RemoteFreeServiceTelemetryTimingStabilityManifestEntry,
    /// Candidate saved output entries.
    pub candidates: Vec<RemoteFreeServiceTelemetryTimingStabilityManifestEntry>,
}

/// Summary status for repeated remote-free service telemetry timing evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetryTimingStabilityStatus {
    /// Every candidate had stable counters and contributed timing evidence.
    Stable,
    /// Some candidates contributed timing evidence and some were discarded.
    Mixed,
    /// Every candidate drifted at the counter layer, so no timings were used.
    CounterDrift,
}

impl RemoteFreeServiceTelemetryTimingStabilityStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Mixed => "mixed",
            Self::CounterDrift => "counter_drift",
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Timing range for one benchmark across baseline plus accepted candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingStabilityRange {
    /// Criterion benchmark label.
    pub benchmark: String,
    /// Number of timing estimates included in this range.
    pub range_runs: usize,
    /// Minimum point estimate in picoseconds.
    pub min_estimate_ps: u128,
    /// Maximum point estimate in picoseconds.
    pub max_estimate_ps: u128,
    /// Max minus min point estimate in picoseconds.
    pub spread_ps: u128,
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_timing_range benchmark={} range_runs={} min_estimate_ps={} max_estimate_ps={} spread_ps={}",
            self.benchmark,
            self.range_runs,
            self.min_estimate_ps,
            self.max_estimate_ps,
            self.spread_ps
        )
    }
}

/// Candidate output discarded from timing ranges due to counter drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingStabilityDiscard {
    /// Candidate run label.
    pub run: String,
    /// Number of drift entries observed in the counter comparison.
    pub drift_entries: usize,
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityDiscard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_timing_discard run={} drift_entries={}",
            self.run, self.drift_entries
        )
    }
}

/// Stability report over repeated remote-free service telemetry outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingStabilityReport {
    /// Summary status.
    pub status: RemoteFreeServiceTelemetryTimingStabilityStatus,
    /// Baseline run label.
    pub baseline: String,
    /// Number of candidate outputs compared with the baseline.
    pub candidate_runs: usize,
    /// Candidate outputs accepted into timing ranges.
    pub accepted_runs: usize,
    /// Candidate outputs discarded from timing ranges.
    pub discarded_runs: usize,
    /// Timing ranges by benchmark.
    pub ranges: Vec<RemoteFreeServiceTelemetryTimingStabilityRange>,
    /// Discarded candidate summaries.
    pub discards: Vec<RemoteFreeServiceTelemetryTimingStabilityDiscard>,
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_timing_stability={} baseline={} candidate_runs={} accepted_runs={} discarded_runs={} timing_ranges={}",
            self.status,
            self.baseline,
            self.candidate_runs,
            self.accepted_runs,
            self.discarded_runs,
            self.ranges.len()
        )
    }
}

/// Error returned when building a repeated-run timing stability report.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetryTimingStabilityError {
    /// No candidate outputs were provided.
    EmptyCandidates,
    /// A baseline or candidate label was empty.
    EmptyLabel,
    /// A run label appeared more than once.
    DuplicateRunLabel(String),
    /// A pairwise comparison failed.
    Compare {
        /// Candidate run label.
        run: String,
        /// Underlying comparison error.
        source: RemoteFreeServiceTelemetrySampleTimingCompareError,
    },
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCandidates => {
                f.write_str("missing remote-free service telemetry candidate runs")
            }
            Self::EmptyLabel => {
                f.write_str("empty remote-free service telemetry timing stability run label")
            }
            Self::DuplicateRunLabel(label) => {
                write!(
                    f,
                    "duplicate remote-free service telemetry timing stability run label: {label}"
                )
            }
            Self::Compare { run, source } => {
                write!(
                    f,
                    "invalid remote-free service telemetry timing stability candidate {run}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryTimingStabilityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Compare { source, .. } => Some(source),
            Self::EmptyCandidates | Self::EmptyLabel | Self::DuplicateRunLabel(_) => None,
        }
    }
}

/// Error returned when parsing a repeated-run timing stability manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetryTimingStabilityManifestParseError {
    /// No data rows were present.
    Empty,
    /// A data row was malformed.
    InvalidLine {
        /// One-based line number.
        line: usize,
        /// Parse failure reason.
        reason: &'static str,
    },
    /// A row used an unsupported role.
    UnknownRole {
        /// One-based line number.
        line: usize,
        /// Role text.
        role: String,
    },
    /// More than one baseline row was present.
    DuplicateBaseline {
        /// One-based line number.
        line: usize,
    },
    /// No baseline row was present.
    MissingBaseline,
    /// No candidate rows were present.
    MissingCandidates,
    /// A run label appeared more than once.
    DuplicateRunLabel {
        /// One-based line number.
        line: usize,
        /// Duplicate label.
        label: String,
    },
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingStabilityManifestParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty remote-free service telemetry stability manifest"),
            Self::InvalidLine { line, reason } => {
                write!(
                    f,
                    "invalid remote-free service telemetry stability manifest line {line}: {reason}"
                )
            }
            Self::UnknownRole { line, role } => {
                write!(
                    f,
                    "unknown remote-free service telemetry stability manifest role on line {line}: {role}"
                )
            }
            Self::DuplicateBaseline { line } => {
                write!(
                    f,
                    "duplicate remote-free service telemetry stability manifest baseline on line {line}"
                )
            }
            Self::MissingBaseline => {
                f.write_str("missing remote-free service telemetry stability manifest baseline")
            }
            Self::MissingCandidates => {
                f.write_str("missing remote-free service telemetry stability manifest candidates")
            }
            Self::DuplicateRunLabel { line, label } => {
                write!(
                    f,
                    "duplicate remote-free service telemetry stability manifest label on line {line}: {label}"
                )
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryTimingStabilityManifestParseError {}

/// Parses a repeated-run timing stability manifest.
///
/// Blank lines and `#` comments are ignored. Each data row must contain
/// exactly three whitespace-separated fields: role, label, and path. The role
/// must be `baseline` or `candidate`.
///
/// # Errors
///
/// Returns an error for empty manifests, malformed rows, unknown roles,
/// duplicate labels, duplicate baselines, missing baseline rows, or missing
/// candidate rows.
pub fn parse_remote_free_service_telemetry_timing_stability_manifest(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryTimingStabilityManifest,
    RemoteFreeServiceTelemetryTimingStabilityManifestParseError,
> {
    let mut saw_data_row = false;
    let mut baseline = None;
    let mut candidates = Vec::new();
    let mut labels = BTreeSet::new();

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        saw_data_row = true;

        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(
                RemoteFreeServiceTelemetryTimingStabilityManifestParseError::InvalidLine {
                    line: line_number,
                    reason: "expected role label path",
                },
            );
        }
        let [role, label, path] = [fields[0], fields[1], fields[2]];
        if label.is_empty() || path.is_empty() {
            return Err(
                RemoteFreeServiceTelemetryTimingStabilityManifestParseError::InvalidLine {
                    line: line_number,
                    reason: "label and path must be non-empty",
                },
            );
        }
        if !labels.insert(label.to_owned()) {
            return Err(
                RemoteFreeServiceTelemetryTimingStabilityManifestParseError::DuplicateRunLabel {
                    line: line_number,
                    label: label.to_owned(),
                },
            );
        }

        let role = match role {
            "baseline" => RemoteFreeServiceTelemetryTimingStabilityManifestRole::Baseline,
            "candidate" => RemoteFreeServiceTelemetryTimingStabilityManifestRole::Candidate,
            _ => {
                return Err(
                    RemoteFreeServiceTelemetryTimingStabilityManifestParseError::UnknownRole {
                        line: line_number,
                        role: role.to_owned(),
                    },
                );
            }
        };
        let entry = RemoteFreeServiceTelemetryTimingStabilityManifestEntry {
            role,
            label: label.to_owned(),
            path: path.to_owned(),
        };

        match role {
            RemoteFreeServiceTelemetryTimingStabilityManifestRole::Baseline => {
                if baseline.replace(entry).is_some() {
                    return Err(
                        RemoteFreeServiceTelemetryTimingStabilityManifestParseError::DuplicateBaseline {
                            line: line_number,
                        },
                    );
                }
            }
            RemoteFreeServiceTelemetryTimingStabilityManifestRole::Candidate => {
                candidates.push(entry);
            }
        }
    }

    if !saw_data_row {
        return Err(RemoteFreeServiceTelemetryTimingStabilityManifestParseError::Empty);
    }
    let baseline = baseline
        .ok_or(RemoteFreeServiceTelemetryTimingStabilityManifestParseError::MissingBaseline)?;
    if candidates.is_empty() {
        return Err(RemoteFreeServiceTelemetryTimingStabilityManifestParseError::MissingCandidates);
    }

    Ok(RemoteFreeServiceTelemetryTimingStabilityManifest {
        baseline,
        candidates,
    })
}

/// Summarizes repeated remote-free service telemetry timing evidence.
///
/// Each candidate is compared with the baseline. Counter-stable candidates
/// contribute their timing estimates to per-benchmark ranges. Counter-drifted
/// candidates are reported as discards and excluded from timing ranges.
///
/// # Errors
///
/// Returns an error when there are no candidates, when run labels are empty or
/// duplicated, or when a counter-stable candidate has missing or malformed
/// timing evidence.
pub fn summarize_remote_free_service_telemetry_timing_stability(
    baseline: RemoteFreeServiceTelemetryTimingStabilityRun<'_>,
    candidates: &[RemoteFreeServiceTelemetryTimingStabilityRun<'_>],
) -> Result<
    RemoteFreeServiceTelemetryTimingStabilityReport,
    RemoteFreeServiceTelemetryTimingStabilityError,
> {
    validate_run_labels(baseline, candidates)?;

    let mut accepted_runs = 0;
    let mut discards = Vec::new();
    let mut ranges = BTreeMap::<String, TimingRangeAccumulator>::new();

    for candidate in candidates {
        let report = compare_remote_free_service_telemetry_sample_outputs_with_timings(
            baseline.output,
            candidate.output,
        )
        .map_err(
            |source| RemoteFreeServiceTelemetryTimingStabilityError::Compare {
                run: candidate.label.to_owned(),
                source,
            },
        )?;

        match report.status {
            RemoteFreeServiceTelemetrySampleTimingCompareStatus::Stable => {
                accepted_runs += 1;
                for delta in report.timing_deltas {
                    record_timing_delta(&mut ranges, &delta);
                }
            }
            RemoteFreeServiceTelemetrySampleTimingCompareStatus::CounterDrift => {
                discards.push(RemoteFreeServiceTelemetryTimingStabilityDiscard {
                    run: candidate.label.to_owned(),
                    drift_entries: report.samples.drifts.len(),
                });
            }
        }
    }

    let ranges = ranges
        .into_iter()
        .map(|(benchmark, range)| range.into_range(benchmark))
        .collect::<Vec<_>>();
    let discarded_runs = discards.len();
    let status = if accepted_runs == candidates.len() {
        RemoteFreeServiceTelemetryTimingStabilityStatus::Stable
    } else if accepted_runs == 0 {
        RemoteFreeServiceTelemetryTimingStabilityStatus::CounterDrift
    } else {
        RemoteFreeServiceTelemetryTimingStabilityStatus::Mixed
    };

    Ok(RemoteFreeServiceTelemetryTimingStabilityReport {
        status,
        baseline: baseline.label.to_owned(),
        candidate_runs: candidates.len(),
        accepted_runs,
        discarded_runs,
        ranges,
        discards,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimingRangeAccumulator {
    baseline_recorded: bool,
    range_runs: usize,
    min_estimate_ps: u128,
    max_estimate_ps: u128,
}

impl TimingRangeAccumulator {
    fn new() -> Self {
        Self {
            baseline_recorded: false,
            range_runs: 0,
            min_estimate_ps: u128::MAX,
            max_estimate_ps: 0,
        }
    }

    fn record(&mut self, estimate_ps: u128) {
        self.range_runs += 1;
        self.min_estimate_ps = self.min_estimate_ps.min(estimate_ps);
        self.max_estimate_ps = self.max_estimate_ps.max(estimate_ps);
    }

    fn into_range(self, benchmark: String) -> RemoteFreeServiceTelemetryTimingStabilityRange {
        RemoteFreeServiceTelemetryTimingStabilityRange {
            benchmark,
            range_runs: self.range_runs,
            min_estimate_ps: self.min_estimate_ps,
            max_estimate_ps: self.max_estimate_ps,
            spread_ps: self.max_estimate_ps - self.min_estimate_ps,
        }
    }
}

fn validate_run_labels(
    baseline: RemoteFreeServiceTelemetryTimingStabilityRun<'_>,
    candidates: &[RemoteFreeServiceTelemetryTimingStabilityRun<'_>],
) -> Result<(), RemoteFreeServiceTelemetryTimingStabilityError> {
    if candidates.is_empty() {
        return Err(RemoteFreeServiceTelemetryTimingStabilityError::EmptyCandidates);
    }
    if baseline.label.is_empty()
        || candidates
            .iter()
            .any(|candidate| candidate.label.is_empty())
    {
        return Err(RemoteFreeServiceTelemetryTimingStabilityError::EmptyLabel);
    }

    let mut labels = BTreeSet::from([baseline.label]);
    for candidate in candidates {
        if !labels.insert(candidate.label) {
            return Err(
                RemoteFreeServiceTelemetryTimingStabilityError::DuplicateRunLabel(
                    candidate.label.to_owned(),
                ),
            );
        }
    }

    Ok(())
}

fn record_timing_delta(
    ranges: &mut BTreeMap<String, TimingRangeAccumulator>,
    delta: &RemoteFreeServiceTelemetryTimingDelta,
) {
    let range = ranges
        .entry(delta.benchmark.clone())
        .or_insert_with(TimingRangeAccumulator::new);

    if !range.baseline_recorded {
        range.record(delta.baseline_estimate_ps);
        range.baseline_recorded = true;
    }
    range.record(delta.candidate_estimate_ps);
}

#[cfg(test)]
mod tests {
    use super::{
        parse_remote_free_service_telemetry_timing_stability_manifest,
        summarize_remote_free_service_telemetry_timing_stability,
        RemoteFreeServiceTelemetryTimingStabilityError,
        RemoteFreeServiceTelemetryTimingStabilityManifestParseError,
        RemoteFreeServiceTelemetryTimingStabilityManifestRole,
        RemoteFreeServiceTelemetryTimingStabilityRun,
        RemoteFreeServiceTelemetryTimingStabilityStatus,
    };
    use crate::remote_free_service_sample_compare::{
        RemoteFreeServiceTelemetrySampleTimingCompareError,
        RemoteFreeServiceTelemetryTimingParseError,
    };

    const APPLY_CONFIRM_SAMPLE: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample","line":"remote_free_service_runtime_apply_confirm_sample submitted_count=768 final_previous_config_present=false","fields":{"submitted_count":768,"drained_count":768,"released_bytes":3145728,"confirm_count":1,"rollback_count":0,"final_previous_config_present":false}}"#;
    const APPLY_CONFIRM_SUMMARY: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample_summary","line":"remote_free_service_runtime_apply_confirm_sample_summary samples=8 policy_drains_mean=12.000","fields":{"samples":8,"policy_drains_mean":12.000}}"#;

    fn sample_output() -> String {
        format!("{APPLY_CONFIRM_SAMPLE}\n{APPLY_CONFIRM_SUMMARY}\n")
    }

    fn timed_output(estimate: &str) -> String {
        format!(
            "{}remote_free_service_runtime_apply_confirm\n                        time:   [56.500 us {estimate} us 57.500 us]\n",
            sample_output()
        )
    }

    fn run<'a>(
        label: &'a str,
        output: &'a str,
    ) -> RemoteFreeServiceTelemetryTimingStabilityRun<'a> {
        RemoteFreeServiceTelemetryTimingStabilityRun { label, output }
    }

    #[test]
    fn parses_timing_stability_manifest() {
        let manifest = parse_remote_free_service_telemetry_timing_stability_manifest(
            r"
            # role label path
            baseline apply-confirm-a evidence/apply-confirm-a.txt

            candidate apply-confirm-b evidence/apply-confirm-b.txt
            candidate apply-confirm-drift evidence/apply-confirm-drift.txt
            ",
        )
        .expect("manifest");

        assert_eq!(
            manifest.baseline.role,
            RemoteFreeServiceTelemetryTimingStabilityManifestRole::Baseline
        );
        assert_eq!(manifest.baseline.label, "apply-confirm-a");
        assert_eq!(manifest.baseline.path, "evidence/apply-confirm-a.txt");
        assert_eq!(manifest.candidates.len(), 2);
        assert_eq!(
            manifest.candidates[0].role,
            RemoteFreeServiceTelemetryTimingStabilityManifestRole::Candidate
        );
        assert_eq!(manifest.candidates[0].label, "apply-confirm-b");
        assert_eq!(
            RemoteFreeServiceTelemetryTimingStabilityManifestRole::Candidate.to_string(),
            "candidate"
        );
    }

    #[test]
    fn rejects_duplicate_manifest_labels() {
        let error = parse_remote_free_service_telemetry_timing_stability_manifest(
            r"
            baseline same a.txt
            candidate same b.txt
            ",
        )
        .expect_err("duplicate label");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::DuplicateRunLabel { .. }
        ));
    }

    #[test]
    fn rejects_duplicate_manifest_baselines() {
        let error = parse_remote_free_service_telemetry_timing_stability_manifest(
            r"
            baseline first a.txt
            baseline second b.txt
            candidate candidate c.txt
            ",
        )
        .expect_err("duplicate baseline");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::DuplicateBaseline { .. }
        ));
    }

    #[test]
    fn rejects_unknown_manifest_roles() {
        let error =
            parse_remote_free_service_telemetry_timing_stability_manifest("control label a.txt\n")
                .expect_err("unknown role");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::UnknownRole { .. }
        ));
    }

    #[test]
    fn rejects_manifest_without_baseline() {
        let error =
            parse_remote_free_service_telemetry_timing_stability_manifest("candidate only a.txt\n")
                .expect_err("missing baseline");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::MissingBaseline
        ));
    }

    #[test]
    fn rejects_manifest_without_candidates() {
        let error =
            parse_remote_free_service_telemetry_timing_stability_manifest("baseline only a.txt\n")
                .expect_err("missing candidates");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::MissingCandidates
        ));
    }

    #[test]
    fn rejects_malformed_manifest_rows() {
        let error = parse_remote_free_service_telemetry_timing_stability_manifest(
            "baseline label path extra\n",
        )
        .expect_err("malformed row");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityManifestParseError::InvalidLine { .. }
        ));
    }

    #[test]
    fn summarizes_stable_candidate_timing_range() {
        let baseline = timed_output("56.600");
        let candidate = timed_output("57.125");
        let report = summarize_remote_free_service_telemetry_timing_stability(
            run("baseline", &baseline),
            &[run("candidate", &candidate)],
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetryTimingStabilityStatus::Stable
        );
        assert_eq!(report.candidate_runs, 1);
        assert_eq!(report.accepted_runs, 1);
        assert_eq!(report.discarded_runs, 0);
        assert_eq!(report.ranges.len(), 1);
        assert_eq!(report.ranges[0].range_runs, 2);
        assert_eq!(report.ranges[0].min_estimate_ps, 56_600_000);
        assert_eq!(report.ranges[0].max_estimate_ps, 57_125_000);
        assert_eq!(report.ranges[0].spread_ps, 525_000);
        assert_eq!(
            report.to_string(),
            "remote_free_service_telemetry_timing_stability=stable baseline=baseline candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1"
        );
        assert_eq!(
            report.ranges[0].to_string(),
            "remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=56600000 max_estimate_ps=57125000 spread_ps=525000"
        );
    }

    #[test]
    fn separates_counter_drifted_candidates_from_timing_ranges() {
        let baseline = timed_output("56.600");
        let candidate = timed_output("57.125");
        let drifted =
            timed_output("55.250").replace("\"submitted_count\":768", "\"submitted_count\":769");
        let report = summarize_remote_free_service_telemetry_timing_stability(
            run("baseline", &baseline),
            &[run("candidate", &candidate), run("drifted", &drifted)],
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetryTimingStabilityStatus::Mixed
        );
        assert_eq!(report.candidate_runs, 2);
        assert_eq!(report.accepted_runs, 1);
        assert_eq!(report.discarded_runs, 1);
        assert_eq!(report.ranges.len(), 1);
        assert_eq!(report.ranges[0].min_estimate_ps, 56_600_000);
        assert_eq!(report.ranges[0].max_estimate_ps, 57_125_000);
        assert_eq!(report.discards.len(), 1);
        assert_eq!(report.discards[0].run, "drifted");
        assert_eq!(report.discards[0].drift_entries, 1);
        assert_eq!(
            report.discards[0].to_string(),
            "remote_free_service_telemetry_timing_discard run=drifted drift_entries=1"
        );
    }

    #[test]
    fn reports_all_drift_candidates_without_timing_ranges() {
        let baseline = timed_output("56.600");
        let drifted =
            timed_output("55.250").replace("\"submitted_count\":768", "\"submitted_count\":769");
        let report = summarize_remote_free_service_telemetry_timing_stability(
            run("baseline", &baseline),
            &[run("drifted", &drifted)],
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetryTimingStabilityStatus::CounterDrift
        );
        assert_eq!(report.accepted_runs, 0);
        assert_eq!(report.discarded_runs, 1);
        assert!(report.ranges.is_empty());
    }

    #[test]
    fn rejects_duplicate_run_labels() {
        let baseline = timed_output("56.600");
        let candidate = timed_output("57.125");
        let error = summarize_remote_free_service_telemetry_timing_stability(
            run("same", &baseline),
            &[run("same", &candidate)],
        )
        .expect_err("duplicate label");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityError::DuplicateRunLabel(_)
        ));
    }

    #[test]
    fn rejects_missing_timing_for_counter_stable_candidate() {
        let baseline = timed_output("56.600");
        let candidate = sample_output();
        let error = summarize_remote_free_service_telemetry_timing_stability(
            run("baseline", &baseline),
            &[run("candidate", &candidate)],
        )
        .expect_err("missing timing");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingStabilityError::Compare {
                source: RemoteFreeServiceTelemetrySampleTimingCompareError::Timings(
                    RemoteFreeServiceTelemetryTimingParseError::MissingBenchmark(_)
                ),
                ..
            }
        ));
    }
}
