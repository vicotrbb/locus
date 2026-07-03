use std::{
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use serde_json::{json, Value};

/// Expected schema for remote-free service telemetry collection summaries.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary.v1";

/// Expected schema for remote-free service telemetry collection summary rollups.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup.v2";

/// Expected schema for remote-free service telemetry collection summary rollup check lines.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check.v1";

/// Expected schema for remote-free service telemetry collection summary rollup check log summaries.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1";

/// Expected schema for remote-free service telemetry collection summary rollup check log summary verification verdicts.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification.v1";

/// Expected schema for rollups of remote-free service telemetry summary verification verdicts.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup.v1";

/// Expected schema for verification verdicts over archived summary verification rollups.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification.v1";

/// Expected schema for summaries of archived summary verification rollup verification verdicts.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary.v1";

/// Expected schema for verification verdicts over archived verifier summaries.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification.v1";

/// Expected schema for rollups of verifier-summary verification verdicts.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1";

/// Expected schema for verification verdicts over archived verifier-summary verification rollups.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1";

/// Parsed remote-free service telemetry collection summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummary {
    /// Collection mode used by the collector.
    pub collection_mode: String,
    /// Stable run id used as the evidence directory name.
    pub run_id: String,
    /// Optional host metadata for the process that captured this bundle.
    pub host: Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
    /// Number of captured output artifacts.
    pub output_count: usize,
    /// Criterion arguments used for benchmark capture.
    pub criterion_args: Vec<String>,
    /// Source entries indexed by the summary.
    pub sources: Vec<RemoteFreeServiceTelemetryCollectionSummarySource>,
    /// Artifact entries indexed by the summary.
    pub artifacts: Vec<RemoteFreeServiceTelemetryCollectionSummaryArtifact>,
}

/// Host metadata associated with a collection summary capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryHost {
    /// Operating system reported by the Rust target.
    pub os: String,
    /// CPU architecture reported by the Rust target.
    pub arch: String,
    /// Hostname visible to the capture process, when available.
    pub hostname: Option<String>,
}

/// One source entry from a collection summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummarySource {
    /// Source role, such as `baseline` or `candidate`.
    pub role: String,
    /// Source label.
    pub label: String,
    /// Benchmark filter or saved-output input path.
    pub input: String,
    /// Relative output artifact path.
    pub artifact: String,
}

/// One artifact entry from a collection summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryArtifact {
    /// Artifact kind, such as `output`, `manifest`, or `validation_summary`.
    pub kind: String,
    /// Optional source role for output artifacts.
    pub role: Option<String>,
    /// Relative artifact path.
    pub path: String,
    /// Expected artifact byte count.
    pub byte_count: u64,
}

/// Filesystem verification report for collection-summary artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryArtifactReport {
    /// Number of artifacts whose byte counts matched.
    pub verified_artifacts: usize,
    /// Sum of verified artifact bytes.
    pub verified_bytes: u64,
}

/// Error returned when scanning for collection summary files.
#[derive(Debug)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryScanError {
    /// Directory that failed to scan.
    pub path: PathBuf,
    /// Underlying I/O error.
    pub source: io::Error,
}

/// Release-check report for a collection summary rollup artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
    /// Rollup artifact path.
    pub path: PathBuf,
    /// Accepted schema string from the artifact.
    pub schema: String,
    /// Exact byte count of the artifact text read for validation.
    pub artifact_bytes: u64,
    /// Stable non-cryptographic fingerprint of the artifact text.
    pub artifact_fingerprint: String,
    /// Number of bundle summaries declared and observed.
    pub summaries: u64,
    /// Number of valid bundle rows declared and observed.
    pub valid_bundles: u64,
    /// Number of drifted saved validation summary rows observed.
    pub drifted_summaries: u64,
    /// Number of missing artifact rows observed.
    pub missing_artifacts: u64,
    /// Number of other failure rows observed.
    pub other_failures: u64,
    /// Number of timing ranges declared and observed.
    pub timing_ranges: u64,
    /// Number of bundle rows in the artifact.
    pub bundles: u64,
    /// Whether the rollup artifact carries rollup refresh host metadata.
    pub rollup_host_present: bool,
    /// Number of bundle rows carrying capture host metadata.
    pub bundle_hosts: u64,
    /// Number of bundle rows without capture host metadata.
    pub bundle_hosts_missing: u64,
}

/// Aggregated report for rollup release-check JSON records found in a saved log.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary {
    /// Number of parsed JSON records.
    pub records: u64,
    /// Number of records with rollup refresh host metadata.
    pub rollup_hosts_present: u64,
    /// Number of records without rollup refresh host metadata.
    pub rollup_hosts_missing: u64,
    /// Number of bundle rows carrying capture host metadata.
    pub bundle_hosts: u64,
    /// Number of bundle rows without capture host metadata.
    pub bundle_hosts_missing: u64,
    /// Number of valid bundle rows across records.
    pub status_valid_bundles: u64,
    /// Number of drifted summary rows across records.
    pub status_drifted_summaries: u64,
    /// Number of missing artifact rows across records.
    pub status_missing_artifacts: u64,
    /// Number of other failure rows across records.
    pub status_other_failures: u64,
}

/// Drift between a recomputed source-log summary and an archived summary JSON line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift {
    /// First field whose recomputed and archived counts differ.
    pub field: &'static str,
    /// Count recomputed from the source rollup-check JSON log.
    pub expected: u64,
    /// Count parsed from the archived summary JSON line.
    pub actual: u64,
}

/// Verification report for an archived summary JSON line checked against source records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification {
    /// Summary recomputed from the source rollup-check JSON records.
    pub expected: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    /// Summary parsed from the archived summary JSON line.
    pub actual: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    /// First counter drift, when the archived summary does not match.
    pub drift: Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
}

/// Aggregated report for saved-log summary verification verdict JSON records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
    /// Number of parsed verdict JSON records.
    pub records: u64,
    /// Number of verdicts whose archived summary matched the source log.
    pub matched: u64,
    /// Number of verdicts whose archived summary drifted from the source log.
    pub drifted: u64,
    /// Number of drifted verdicts whose first drift was `records`.
    pub drift_records: u64,
    /// Number of drifted verdicts whose first drift was `rollup_hosts_present`.
    pub drift_rollup_hosts_present: u64,
    /// Number of drifted verdicts whose first drift was `rollup_hosts_missing`.
    pub drift_rollup_hosts_missing: u64,
    /// Number of drifted verdicts whose first drift was `bundle_hosts`.
    pub drift_bundle_hosts: u64,
    /// Number of drifted verdicts whose first drift was `bundle_hosts_missing`.
    pub drift_bundle_hosts_missing: u64,
    /// Number of drifted verdicts whose first drift was `status_valid_bundles`.
    pub drift_status_valid_bundles: u64,
    /// Number of drifted verdicts whose first drift was `status_drifted_summaries`.
    pub drift_status_drifted_summaries: u64,
    /// Number of drifted verdicts whose first drift was `status_missing_artifacts`.
    pub drift_status_missing_artifacts: u64,
    /// Number of drifted verdicts whose first drift was `status_other_failures`.
    pub drift_status_other_failures: u64,
}

/// Drift between a recomputed verdict rollup and an archived verdict rollup JSON line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift {
    /// First field whose recomputed and archived counts differ.
    pub field: &'static str,
    /// Count recomputed from the saved verdict JSON log.
    pub expected: u64,
    /// Count parsed from the archived verdict rollup JSON line.
    pub actual: u64,
}

/// Verification report for an archived verdict rollup JSON line checked against source verdicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification
{
    /// Rollup recomputed from the saved verdict JSON records.
    pub expected:
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    /// Rollup parsed from the archived verdict rollup JSON line.
    pub actual: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    /// First counter drift, when the archived verdict rollup does not match.
    pub drift: Option<
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift,
    >,
}

/// Aggregated report for saved verdict rollup verification JSON records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary
{
    /// Number of parsed rollup-verification JSON records.
    pub records: u64,
    /// Number of artifacts whose archived verdict rollup matched source verdicts.
    pub matched: u64,
    /// Number of artifacts whose archived verdict rollup drifted from source verdicts.
    pub drifted: u64,
    /// Number of drifted artifacts whose first drift was `records`.
    pub drift_records: u64,
    /// Number of drifted artifacts whose first drift was `matched`.
    pub drift_matched: u64,
    /// Number of drifted artifacts whose first drift was `drifted`.
    pub drift_drifted: u64,
    /// Number of drifted artifacts whose first drift was `drift_records`.
    pub drift_drift_records: u64,
    /// Number of drifted artifacts whose first drift was `drift_rollup_hosts_present`.
    pub drift_drift_rollup_hosts_present: u64,
    /// Number of drifted artifacts whose first drift was `drift_rollup_hosts_missing`.
    pub drift_drift_rollup_hosts_missing: u64,
    /// Number of drifted artifacts whose first drift was `drift_bundle_hosts`.
    pub drift_drift_bundle_hosts: u64,
    /// Number of drifted artifacts whose first drift was `drift_bundle_hosts_missing`.
    pub drift_drift_bundle_hosts_missing: u64,
    /// Number of drifted artifacts whose first drift was `drift_status_valid_bundles`.
    pub drift_drift_status_valid_bundles: u64,
    /// Number of drifted artifacts whose first drift was `drift_status_drifted_summaries`.
    pub drift_drift_status_drifted_summaries: u64,
    /// Number of drifted artifacts whose first drift was `drift_status_missing_artifacts`.
    pub drift_drift_status_missing_artifacts: u64,
    /// Number of drifted artifacts whose first drift was `drift_status_other_failures`.
    pub drift_drift_status_other_failures: u64,
}

/// Drift between a recomputed verifier summary and an archived verifier summary JSON line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift
{
    /// First field whose recomputed and archived counts differ.
    pub field: &'static str,
    /// Count recomputed from the saved verifier JSON records.
    pub expected: u64,
    /// Count parsed from the archived verifier-summary JSON line.
    pub actual: u64,
}

/// Verification report for an archived verifier summary checked against source verifier records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification
{
    /// Summary recomputed from the saved verifier JSON records.
    pub expected: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    /// Summary parsed from the archived verifier-summary JSON line.
    pub actual: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    /// First counter drift, when the archived verifier summary does not match.
    pub drift: Option<
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
}

/// Aggregated report for saved verifier-summary verification JSON records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup
{
    /// Cohort counters for verifier-summary verification verdicts.
    pub summary: RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
}

/// Verification report for an archived verifier-summary verification rollup checked against source records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification
{
    /// Rollup recomputed from the saved verifier-summary verification records.
    pub expected:
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    /// Rollup parsed from the archived verifier-summary verification rollup JSON line.
    pub actual:
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    /// First counter drift, when the archived rollup does not match.
    pub drift: Option<
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
}

impl RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification {
    /// Returns true when the archived summary matches the recomputed source-log summary.
    #[must_use]
    pub fn is_matched(&self) -> bool {
        self.drift.is_none()
    }

    /// Stable status label for compact logs and JSON.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        if self.is_matched() {
            "matched"
        } else {
            "drifted"
        }
    }
}

impl
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification
{
    /// Returns true when the archived verdict rollup matches the recomputed source-log rollup.
    #[must_use]
    pub fn is_matched(&self) -> bool {
        self.drift.is_none()
    }

    /// Stable status label for compact logs and JSON.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        if self.is_matched() {
            "matched"
        } else {
            "drifted"
        }
    }
}

impl
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification
{
    /// Returns true when the archived verifier summary matches the recomputed source-log summary.
    #[must_use]
    pub fn is_matched(&self) -> bool {
        self.drift.is_none()
    }

    /// Stable status label for compact logs and JSON.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        if self.is_matched() {
            "matched"
        } else {
            "drifted"
        }
    }
}

impl
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification
{
    /// Returns true when the archived rollup matches the recomputed source-log rollup.
    #[must_use]
    pub fn is_matched(&self) -> bool {
        self.drift.is_none()
    }

    /// Stable status label for compact logs and JSON.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        if self.is_matched() {
            "matched"
        } else {
            "drifted"
        }
    }
}

/// Collection summary rollup data used to write schema v2 artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollup {
    /// Evidence root associated with the rollup.
    pub root: PathBuf,
    /// Optional host metadata for the process that refreshed this rollup.
    pub host: Option<RemoteFreeServiceTelemetryCollectionSummaryRollupHost>,
    /// Number of discovered collection summaries.
    pub summaries: u64,
    /// Number of valid bundle rows.
    pub valid_bundles: u64,
    /// Number of drifted saved validation summaries.
    pub drifted_summaries: u64,
    /// Number of bundles with missing artifacts.
    pub missing_artifacts: u64,
    /// Number of other validation failures.
    pub other_failures: u64,
    /// Total timing range rows across valid bundles.
    pub timing_ranges: u64,
    /// Per-bundle rollup rows.
    pub bundles: Vec<RemoteFreeServiceTelemetryCollectionSummaryRollupBundle>,
}

/// Host metadata associated with a collection summary rollup refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupHost {
    /// Operating system reported by the Rust target.
    pub os: String,
    /// CPU architecture reported by the Rust target.
    pub arch: String,
    /// Hostname visible to the refresh process, when available.
    pub hostname: Option<String>,
}

/// Caller-provided result for one collection summary path during rollup build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
    /// Run id when the summary could be parsed.
    pub run_id: Option<String>,
    /// Capture host metadata when the summary could be parsed and carried it.
    pub host: Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
    /// Bundle validation status.
    pub status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
    /// Number of timing ranges for this bundle.
    pub timing_ranges: usize,
}

/// One bundle row in a collection summary rollup artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummaryRollupBundle {
    /// Summary path, usually relative to the rollup root.
    pub summary: String,
    /// Run id when the summary could be parsed.
    pub run_id: Option<String>,
    /// Capture host metadata for the bundle, when available.
    pub host: Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
    /// Bundle validation status.
    pub status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
    /// Number of timing ranges for this bundle.
    pub timing_ranges: u64,
}

/// Stable status labels for collection summary rollup bundle rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus {
    /// Bundle passed validation.
    Valid,
    /// Saved validation summary drifted from a fresh computation.
    DriftedSummary,
    /// One or more artifacts listed by the summary were missing.
    MissingArtifact,
    /// Bundle failed for a different validation reason.
    OtherFailure,
}

impl RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus {
    /// Stable status label used in rollup artifacts.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::DriftedSummary => "drifted_summary",
            Self::MissingArtifact => "missing_artifact",
            Self::OtherFailure => "other_failure",
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryArtifactReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts={} verified_bytes={}",
            self.verified_artifacts, self.verified_bytes
        )
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to scan remote-free service telemetry collection summary directory {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryCollectionSummaryScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup_check=ok path={} schema={} artifact_bytes={} artifact_fingerprint={} summaries={} valid_bundles={} timing_ranges={} bundles={} rollup_host_present={} bundle_hosts={} bundle_hosts_missing={} status_valid_bundles={} status_drifted_summaries={} status_missing_artifacts={} status_other_failures={}",
            self.path.display(),
            self.schema,
            self.artifact_bytes,
            self.artifact_fingerprint,
            self.summaries,
            self.valid_bundles,
            self.timing_ranges,
            self.bundles,
            self.rollup_host_present,
            self.bundle_hosts,
            self.bundle_hosts_missing,
            self.valid_bundles,
            self.drifted_summaries,
            self.missing_artifacts,
            self.other_failures
        )
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup_check_log=ok records={} rollup_hosts_present={} rollup_hosts_missing={} bundle_hosts={} bundle_hosts_missing={} status_valid_bundles={} status_drifted_summaries={} status_missing_artifacts={} status_other_failures={}",
            self.records,
            self.rollup_hosts_present,
            self.rollup_hosts_missing,
            self.bundle_hosts,
            self.bundle_hosts_missing,
            self.status_valid_bundles,
            self.status_drifted_summaries,
            self.status_missing_artifacts,
            self.status_other_failures
        )
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.drift {
            Some(drift) => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification=drifted field={} expected={} actual={} expected_records={} actual_records={}",
                drift.field,
                drift.expected,
                drift.actual,
                self.expected.records,
                self.actual.records
            ),
            None => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification=matched records={} rollup_hosts_present={} rollup_hosts_missing={} bundle_hosts={} bundle_hosts_missing={} status_valid_bundles={} status_drifted_summaries={} status_missing_artifacts={} status_other_failures={}",
                self.actual.records,
                self.actual.rollup_hosts_present,
                self.actual.rollup_hosts_missing,
                self.actual.bundle_hosts,
                self.actual.bundle_hosts_missing,
                self.actual.status_valid_bundles,
                self.actual.status_drifted_summaries,
                self.actual.status_missing_artifacts,
                self.actual.status_other_failures
            ),
        }
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup=ok records={} matched={} drifted={} drift_records={} drift_rollup_hosts_present={} drift_rollup_hosts_missing={} drift_bundle_hosts={} drift_bundle_hosts_missing={} drift_status_valid_bundles={} drift_status_drifted_summaries={} drift_status_missing_artifacts={} drift_status_other_failures={}",
            self.records,
            self.matched,
            self.drifted,
            self.drift_records,
            self.drift_rollup_hosts_present,
            self.drift_rollup_hosts_missing,
            self.drift_bundle_hosts,
            self.drift_bundle_hosts_missing,
            self.drift_status_valid_bundles,
            self.drift_status_drifted_summaries,
            self.drift_status_missing_artifacts,
            self.drift_status_other_failures
        )
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.drift {
            Some(drift) => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=drifted field={} expected={} actual={} expected_records={} actual_records={}",
                drift.field,
                drift.expected,
                drift.actual,
                self.expected.records,
                self.actual.records
            ),
            None => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=matched records={} matched={} drifted={} drift_records={} drift_rollup_hosts_present={} drift_rollup_hosts_missing={} drift_bundle_hosts={} drift_bundle_hosts_missing={} drift_status_valid_bundles={} drift_status_drifted_summaries={} drift_status_missing_artifacts={} drift_status_other_failures={}",
                self.actual.records,
                self.actual.matched,
                self.actual.drifted,
                self.actual.drift_records,
                self.actual.drift_rollup_hosts_present,
                self.actual.drift_rollup_hosts_missing,
                self.actual.drift_bundle_hosts,
                self.actual.drift_bundle_hosts_missing,
                self.actual.drift_status_valid_bundles,
                self.actual.drift_status_drifted_summaries,
                self.actual.drift_status_missing_artifacts,
                self.actual.drift_status_other_failures
            ),
        }
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary=ok records={} matched={} drifted={} drift_records={} drift_matched={} drift_drifted={} drift_drift_records={} drift_drift_rollup_hosts_present={} drift_drift_rollup_hosts_missing={} drift_drift_bundle_hosts={} drift_drift_bundle_hosts_missing={} drift_drift_status_valid_bundles={} drift_drift_status_drifted_summaries={} drift_drift_status_missing_artifacts={} drift_drift_status_other_failures={}",
            self.records,
            self.matched,
            self.drifted,
            self.drift_records,
            self.drift_matched,
            self.drift_drifted,
            self.drift_drift_records,
            self.drift_drift_rollup_hosts_present,
            self.drift_drift_rollup_hosts_missing,
            self.drift_drift_bundle_hosts,
            self.drift_drift_bundle_hosts_missing,
            self.drift_drift_status_valid_bundles,
            self.drift_drift_status_drifted_summaries,
            self.drift_drift_status_missing_artifacts,
            self.drift_drift_status_other_failures
        )
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.drift {
            Some(drift) => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=drifted field={} expected={} actual={} expected_records={} actual_records={}",
                drift.field,
                drift.expected,
                drift.actual,
                self.expected.records,
                self.actual.records
            ),
            None => write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=matched records={} matched={} drifted={} drift_records={} drift_matched={} drift_drifted={} drift_drift_records={} drift_drift_rollup_hosts_present={} drift_drift_rollup_hosts_missing={} drift_drift_bundle_hosts={} drift_drift_bundle_hosts_missing={} drift_drift_status_valid_bundles={} drift_drift_status_drifted_summaries={} drift_drift_status_missing_artifacts={} drift_drift_status_other_failures={}",
                self.actual.records,
                self.actual.matched,
                self.actual.drifted,
                self.actual.drift_records,
                self.actual.drift_matched,
                self.actual.drift_drifted,
                self.actual.drift_drift_records,
                self.actual.drift_drift_rollup_hosts_present,
                self.actual.drift_drift_rollup_hosts_missing,
                self.actual.drift_drift_bundle_hosts,
                self.actual.drift_drift_bundle_hosts_missing,
                self.actual.drift_drift_status_valid_bundles,
                self.actual.drift_drift_status_drifted_summaries,
                self.actual.drift_drift_status_missing_artifacts,
                self.actual.drift_drift_status_other_failures
            ),
        }
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let summary = &self.summary;
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records={} matched={} drifted={} drift_records={} drift_matched={} drift_drifted={} drift_drift_records={} drift_drift_rollup_hosts_present={} drift_drift_rollup_hosts_missing={} drift_drift_bundle_hosts={} drift_drift_bundle_hosts_missing={} drift_drift_status_valid_bundles={} drift_drift_status_drifted_summaries={} drift_drift_status_missing_artifacts={} drift_drift_status_other_failures={}",
            summary.records,
            summary.matched,
            summary.drifted,
            summary.drift_records,
            summary.drift_matched,
            summary.drift_drifted,
            summary.drift_drift_records,
            summary.drift_drift_rollup_hosts_present,
            summary.drift_drift_rollup_hosts_missing,
            summary.drift_drift_bundle_hosts,
            summary.drift_drift_bundle_hosts_missing,
            summary.drift_drift_status_valid_bundles,
            summary.drift_drift_status_drifted_summaries,
            summary.drift_drift_status_missing_artifacts,
            summary.drift_drift_status_other_failures
        )
    }
}

impl fmt::Display
    for RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(drift) = &self.drift {
            write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field={} expected={} actual={} expected_records={} actual_records={}",
                drift.field,
                drift.expected,
                drift.actual,
                self.expected.summary.records,
                self.actual.summary.records
            )
        } else {
            let summary = &self.actual.summary;
            write!(
                f,
                "remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records={} matched={} drifted={} drift_records={} drift_matched={} drift_drifted={} drift_drift_records={} drift_drift_rollup_hosts_present={} drift_drift_rollup_hosts_missing={} drift_drift_bundle_hosts={} drift_drift_bundle_hosts_missing={} drift_drift_status_valid_bundles={} drift_drift_status_drifted_summaries={} drift_drift_status_missing_artifacts={} drift_drift_status_other_failures={}",
                summary.records,
                summary.matched,
                summary.drifted,
                summary.drift_records,
                summary.drift_matched,
                summary.drift_drifted,
                summary.drift_drift_records,
                summary.drift_drift_rollup_hosts_present,
                summary.drift_drift_rollup_hosts_missing,
                summary.drift_drift_bundle_hosts,
                summary.drift_drift_bundle_hosts_missing,
                summary.drift_drift_status_valid_bundles,
                summary.drift_drift_status_drifted_summaries,
                summary.drift_drift_status_missing_artifacts,
                summary.drift_drift_status_other_failures
            )
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryRollup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_rollup root={} summaries={} valid_bundles={} drifted_summaries={} missing_artifacts={} other_failures={} timing_ranges={}",
            self.root.display(),
            self.summaries,
            self.valid_bundles,
            self.drifted_summaries,
            self.missing_artifacts,
            self.other_failures,
            self.timing_ranges
        )
    }
}

/// Error returned when parsing or validating a collection summary.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetryCollectionSummaryError {
    /// JSON parsing failed.
    Json(serde_json::Error),
    /// Required field was missing.
    MissingField(&'static str),
    /// Field had the wrong JSON type.
    InvalidFieldType(&'static str),
    /// Schema was not the expected summary schema.
    UnexpectedSchema(String),
    /// Summary output count did not match output artifact count.
    OutputCountMismatch {
        /// Declared output count.
        declared: usize,
        /// Output artifacts listed in the summary.
        artifacts: usize,
    },
    /// Summary does not list a manifest artifact.
    MissingManifestArtifact,
    /// Summary does not list a validation-summary artifact.
    MissingValidationSummaryArtifact,
    /// Artifact path was absolute or escaped the summary directory.
    InvalidArtifactPath(String),
    /// Filesystem access failed while validating an artifact.
    Io {
        /// Artifact path from the summary.
        path: String,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Filesystem byte count did not match the summary entry.
    ByteCountMismatch {
        /// Artifact path from the summary.
        path: String,
        /// Expected byte count.
        expected: u64,
        /// Actual byte count.
        actual: u64,
    },
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(source) => write!(
                f,
                "invalid remote-free service telemetry collection summary JSON: {source}"
            ),
            Self::MissingField(field) => write!(
                f,
                "missing remote-free service telemetry collection summary field: {field}"
            ),
            Self::InvalidFieldType(field) => write!(
                f,
                "invalid remote-free service telemetry collection summary field type: {field}"
            ),
            Self::UnexpectedSchema(schema) => write!(
                f,
                "unexpected remote-free service telemetry collection summary schema: {schema}"
            ),
            Self::OutputCountMismatch {
                declared,
                artifacts,
            } => write!(
                f,
                "remote-free service telemetry collection summary output count mismatch: declared={declared} artifacts={artifacts}"
            ),
            Self::MissingManifestArtifact => f.write_str(
                "missing remote-free service telemetry collection summary manifest artifact",
            ),
            Self::MissingValidationSummaryArtifact => f.write_str(
                "missing remote-free service telemetry collection summary validation-summary artifact",
            ),
            Self::InvalidArtifactPath(path) => write!(
                f,
                "invalid remote-free service telemetry collection summary artifact path: {path}"
            ),
            Self::Io { path, source } => write!(
                f,
                "failed to read remote-free service telemetry collection summary artifact {path}: {source}"
            ),
            Self::ByteCountMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "remote-free service telemetry collection summary artifact byte count mismatch for {path}: expected={expected} actual={actual}"
            ),
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryCollectionSummaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::MissingField(_)
            | Self::InvalidFieldType(_)
            | Self::UnexpectedSchema(_)
            | Self::OutputCountMismatch { .. }
            | Self::MissingManifestArtifact
            | Self::MissingValidationSummaryArtifact
            | Self::InvalidArtifactPath(_)
            | Self::ByteCountMismatch { .. } => None,
        }
    }
}

/// Error returned when building a collection summary directory rollup.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError {
    /// Directory scanning failed.
    Scan(RemoteFreeServiceTelemetryCollectionSummaryScanError),
    /// A count did not fit in the exported rollup representation.
    CountOverflow(&'static str),
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scan(source) => write!(
                f,
                "failed to build remote-free service telemetry collection summary directory rollup: {source}"
            ),
            Self::CountOverflow(field) => write!(
                f,
                "remote-free service telemetry collection summary directory rollup count overflow: {field}"
            ),
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Scan(source) => Some(source),
            Self::CountOverflow(_) => None,
        }
    }
}

/// Error returned when validating a collection summary rollup artifact.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetryCollectionSummaryRollupError {
    /// Filesystem access failed while reading the rollup artifact.
    Io {
        /// Rollup artifact path.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Filesystem access failed while writing the rollup artifact.
    WriteIo {
        /// Rollup artifact path.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// JSON parsing failed.
    Json(serde_json::Error),
    /// JSON serialization failed.
    Serialize(serde_json::Error),
    /// Required field was missing.
    MissingField(&'static str),
    /// Field had the wrong JSON type.
    InvalidFieldType(&'static str),
    /// Schema was not the expected rollup schema.
    UnexpectedSchema(String),
    /// Bundle row status was not recognized.
    UnknownBundleStatus {
        /// Summary path from the bundle row.
        summary: String,
        /// Status value from the bundle row.
        status: String,
    },
    /// Declared aggregate count did not match bundle rows.
    CountDrift {
        /// Aggregate field name.
        field: &'static str,
        /// Declared aggregate count.
        expected: u64,
        /// Count computed from bundle rows.
        actual: u64,
    },
    /// A JSON release-check field did not match its duplicated source field.
    JsonFieldDrift {
        /// Field name.
        field: &'static str,
        /// Expected value.
        expected: String,
        /// Actual value.
        actual: String,
    },
    /// Aggregating parsed records overflowed a counter.
    CountOverflow(&'static str),
    /// The artifact contains failed bundle rows.
    FailedBundles {
        /// Number of valid bundle rows.
        valid_bundles: u64,
        /// Number of drifted summary rows.
        drifted_summaries: u64,
        /// Number of missing artifact rows.
        missing_artifacts: u64,
        /// Number of other failure rows.
        other_failures: u64,
    },
}

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryRollupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to read remote-free service telemetry collection summary rollup artifact {}: {source}",
                    path.display()
                )
            }
            Self::WriteIo { path, source } => {
                write!(
                    f,
                    "failed to write remote-free service telemetry collection summary rollup artifact {}: {source}",
                    path.display()
                )
            }
            Self::Json(source) => write!(
                f,
                "invalid remote-free service telemetry collection summary rollup JSON: {source}"
            ),
            Self::Serialize(source) => write!(
                f,
                "failed to serialize remote-free service telemetry collection summary rollup JSON: {source}"
            ),
            Self::MissingField(field) => write!(
                f,
                "missing remote-free service telemetry collection summary rollup field: {field}"
            ),
            Self::InvalidFieldType(field) => write!(
                f,
                "invalid remote-free service telemetry collection summary rollup field type: {field}"
            ),
            Self::UnexpectedSchema(schema) => write!(
                f,
                "unexpected remote-free service telemetry collection summary rollup schema: {schema}"
            ),
            Self::UnknownBundleStatus { summary, status } => write!(
                f,
                "unknown remote-free service telemetry collection summary rollup bundle status: summary={summary} status={status}"
            ),
            Self::CountDrift {
                field,
                expected,
                actual,
            } => write!(
                f,
                "remote-free service telemetry collection summary rollup count drift: field={field} expected={expected} actual={actual}"
            ),
            Self::JsonFieldDrift {
                field,
                expected,
                actual,
            } => write!(
                f,
                "remote-free service telemetry collection summary rollup JSON field drift: field={field} expected={expected} actual={actual}"
            ),
            Self::CountOverflow(field) => write!(
                f,
                "remote-free service telemetry collection summary rollup count overflow: {field}"
            ),
            Self::FailedBundles {
                valid_bundles,
                drifted_summaries,
                missing_artifacts,
                other_failures,
            } => write!(
                f,
                "remote-free service telemetry collection summary rollup contains failed bundles: valid_bundles={valid_bundles} drifted_summaries={drifted_summaries} missing_artifacts={missing_artifacts} other_failures={other_failures}"
            ),
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryCollectionSummaryRollupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::WriteIo { source, .. } => Some(source),
            Self::Json(source) | Self::Serialize(source) => Some(source),
            Self::MissingField(_)
            | Self::InvalidFieldType(_)
            | Self::UnexpectedSchema(_)
            | Self::UnknownBundleStatus { .. }
            | Self::CountDrift { .. }
            | Self::JsonFieldDrift { .. }
            | Self::CountOverflow(_)
            | Self::FailedBundles { .. } => None,
        }
    }
}

/// Parses a remote-free service telemetry collection summary.
///
/// # Errors
///
/// Returns an error when JSON is invalid, required fields are missing or have
/// the wrong type, the schema is unsupported, or the declared output count does
/// not match the number of listed output artifacts.
pub fn parse_remote_free_service_telemetry_collection_summary(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummary,
    RemoteFreeServiceTelemetryCollectionSummaryError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryError::Json)?;
    let schema = required_str(&value, "schema")?;
    if schema != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_SCHEMA {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryError::UnexpectedSchema(schema.to_owned()),
        );
    }

    let collection_mode = required_str(&value, "collection_mode")?.to_owned();
    let run_id = required_str(&value, "run_id")?.to_owned();
    let host = optional_summary_host(&value)?;
    let output_count = required_usize(&value, "output_count")?;
    let criterion_args = required_string_array(&value, "criterion_args")?;
    let sources = required_array(&value, "sources")?
        .iter()
        .map(parse_source)
        .collect::<Result<Vec<_>, _>>()?;
    let artifacts = required_array(&value, "artifacts")?
        .iter()
        .map(parse_artifact)
        .collect::<Result<Vec<_>, _>>()?;

    let output_artifacts = artifacts
        .iter()
        .filter(|artifact| artifact.kind == "output")
        .count();
    if output_count != output_artifacts {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryError::OutputCountMismatch {
                declared: output_count,
                artifacts: output_artifacts,
            },
        );
    }

    Ok(RemoteFreeServiceTelemetryCollectionSummary {
        collection_mode,
        run_id,
        host,
        output_count,
        criterion_args,
        sources,
        artifacts,
    })
}

/// Verifies all artifact byte counts listed in a collection summary.
///
/// # Errors
///
/// Returns an error when an artifact path is unsafe, metadata cannot be read,
/// or a byte count differs from the filesystem.
pub fn verify_remote_free_service_telemetry_collection_summary_artifacts(
    summary_path: &Path,
    summary: &RemoteFreeServiceTelemetryCollectionSummary,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryArtifactReport,
    RemoteFreeServiceTelemetryCollectionSummaryError,
> {
    let base_dir = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let mut verified_bytes = 0u64;

    for artifact in &summary.artifacts {
        let path = resolve_summary_artifact_path(base_dir, &artifact.path)?;
        let metadata = fs::metadata(&path).map_err(|source| {
            RemoteFreeServiceTelemetryCollectionSummaryError::Io {
                path: artifact.path.clone(),
                source,
            }
        })?;
        let actual = metadata.len();
        if actual != artifact.byte_count {
            return Err(
                RemoteFreeServiceTelemetryCollectionSummaryError::ByteCountMismatch {
                    path: artifact.path.clone(),
                    expected: artifact.byte_count,
                    actual,
                },
            );
        }
        verified_bytes = verified_bytes.saturating_add(actual);
    }

    Ok(RemoteFreeServiceTelemetryCollectionSummaryArtifactReport {
        verified_artifacts: summary.artifacts.len(),
        verified_bytes,
    })
}

/// Resolves the manifest artifact listed in a collection summary.
///
/// # Errors
///
/// Returns an error when no manifest artifact is present or when its path is
/// unsafe.
pub fn resolve_remote_free_service_telemetry_collection_summary_manifest_path(
    summary_path: &Path,
    summary: &RemoteFreeServiceTelemetryCollectionSummary,
) -> Result<PathBuf, RemoteFreeServiceTelemetryCollectionSummaryError> {
    resolve_artifact_path_by_kind(
        summary_path,
        summary,
        "manifest",
        RemoteFreeServiceTelemetryCollectionSummaryError::MissingManifestArtifact,
    )
}

/// Resolves the validation-summary artifact listed in a collection summary.
///
/// # Errors
///
/// Returns an error when no validation-summary artifact is present or when its
/// path is unsafe.
pub fn resolve_remote_free_service_telemetry_collection_summary_validation_summary_path(
    summary_path: &Path,
    summary: &RemoteFreeServiceTelemetryCollectionSummary,
) -> Result<PathBuf, RemoteFreeServiceTelemetryCollectionSummaryError> {
    resolve_artifact_path_by_kind(
        summary_path,
        summary,
        "validation_summary",
        RemoteFreeServiceTelemetryCollectionSummaryError::MissingValidationSummaryArtifact,
    )
}

/// Recursively finds remote-free service telemetry collection summaries.
///
/// # Errors
///
/// Returns an error when a directory in the evidence tree cannot be read.
pub fn collect_remote_free_service_telemetry_collection_summary_paths(
    root: &Path,
) -> Result<Vec<PathBuf>, RemoteFreeServiceTelemetryCollectionSummaryScanError> {
    let mut summary_paths = Vec::new();
    collect_summary_paths_recursive(root, &mut summary_paths)?;
    summary_paths.sort();
    Ok(summary_paths)
}

/// Builds a directory rollup from caller-provided per-summary validation.
///
/// # Errors
///
/// Returns an error when the evidence root cannot be scanned or a count does
/// not fit the exported rollup representation.
pub fn build_remote_free_service_telemetry_collection_summary_directory_rollup(
    root: &Path,
    mut validate_summary: impl FnMut(
        &Path,
    ) -> RemoteFreeServiceTelemetryCollectionSummaryBundleValidation,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollup,
    RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError,
> {
    let summary_paths = collect_remote_free_service_telemetry_collection_summary_paths(root)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError::Scan)?;

    let mut rollup = RemoteFreeServiceTelemetryCollectionSummaryRollup {
        root: root.to_path_buf(),
        host: None,
        summaries: usize_to_u64(summary_paths.len(), "summaries")?,
        valid_bundles: 0,
        drifted_summaries: 0,
        missing_artifacts: 0,
        other_failures: 0,
        timing_ranges: 0,
        bundles: Vec::with_capacity(summary_paths.len()),
    };

    for summary_path in summary_paths {
        let validation = validate_summary(&summary_path);
        let timing_ranges = usize_to_u64(validation.timing_ranges, "timing_ranges")?;
        match validation.status {
            RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid => {
                rollup.valid_bundles =
                    checked_add_rollup_count(rollup.valid_bundles, 1, "valid_bundles")?;
                rollup.timing_ranges =
                    checked_add_rollup_count(rollup.timing_ranges, timing_ranges, "timing_ranges")?;
            }
            RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::DriftedSummary => {
                rollup.drifted_summaries =
                    checked_add_rollup_count(rollup.drifted_summaries, 1, "drifted_summaries")?;
            }
            RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::MissingArtifact => {
                rollup.missing_artifacts =
                    checked_add_rollup_count(rollup.missing_artifacts, 1, "missing_artifacts")?;
            }
            RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::OtherFailure => {
                rollup.other_failures =
                    checked_add_rollup_count(rollup.other_failures, 1, "other_failures")?;
            }
        }

        let summary = summary_path
            .strip_prefix(root)
            .unwrap_or(&summary_path)
            .display()
            .to_string();
        rollup
            .bundles
            .push(RemoteFreeServiceTelemetryCollectionSummaryRollupBundle {
                summary,
                run_id: validation.run_id,
                host: validation.host,
                status: validation.status,
                timing_ranges,
            });
    }

    Ok(rollup)
}

/// Writes a schema v2 collection summary rollup artifact at the evidence root.
///
/// # Errors
///
/// Returns an error when the artifact cannot be serialized or written.
pub fn write_remote_free_service_telemetry_collection_summary_rollup_artifact(
    root: &Path,
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollup,
) -> Result<PathBuf, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let bundles = rollup
        .bundles
        .iter()
        .map(|bundle| {
            let mut bundle_json = json!({
                "summary": bundle.summary.as_str(),
                "run_id": bundle.run_id.as_deref(),
                "status": bundle.status.as_str(),
                "timing_ranges": bundle.timing_ranges,
            });
            if let Some(host) = &bundle.host {
                if let Some(object) = bundle_json.as_object_mut() {
                    object.insert("host".to_owned(), summary_host_json(host));
                }
            }
            bundle_json
        })
        .collect::<Vec<_>>();
    let mut artifact = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_SCHEMA,
        "root": rollup.root.display().to_string(),
        "summaries": rollup.summaries,
        "valid_bundles": rollup.valid_bundles,
        "drifted_summaries": rollup.drifted_summaries,
        "missing_artifacts": rollup.missing_artifacts,
        "other_failures": rollup.other_failures,
        "timing_ranges": rollup.timing_ranges,
        "bundles": bundles,
    });
    if let Some(host) = &rollup.host {
        if let Some(object) = artifact.as_object_mut() {
            object.insert(
                "host".to_owned(),
                json!({
                    "os": host.os.as_str(),
                    "arch": host.arch.as_str(),
                    "hostname": host.hostname.as_deref(),
                }),
            );
        }
    }
    let path = root.join("collection-summary-rollup.json");
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&artifact)
            .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)?
    );
    fs::write(&path, text).map_err(|source| {
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::WriteIo {
            path: path.clone(),
            source,
        }
    })?;
    Ok(path)
}

/// Validates a remote-free service telemetry collection summary rollup artifact.
///
/// # Errors
///
/// Returns an error when the artifact cannot be read, JSON is invalid, the
/// schema is unsupported, aggregate counts drift from bundle rows, or any
/// bundle row reports a failed status.
pub fn validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
    path: &Path,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let artifact_text = fs::read_to_string(path).map_err(|source| {
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::Io {
            path: path.to_path_buf(),
            source,
        }
    })?;
    let artifact_bytes = artifact_text.len().try_into().map_err(|_| {
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType("artifact_bytes")
    })?;
    let artifact_fingerprint = rollup_artifact_fingerprint(&artifact_text);
    let artifact = serde_json::from_str::<Value>(&artifact_text)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    let schema = rollup_required_str(&artifact, "schema")?;
    if schema != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_SCHEMA {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }

    let bundles = rollup_required_array(&artifact, "bundles")?;
    let expected_summaries = rollup_required_u64(&artifact, "summaries")?;
    let expected_valid_bundles = rollup_required_u64(&artifact, "valid_bundles")?;
    let expected_drifted_summaries = rollup_required_u64(&artifact, "drifted_summaries")?;
    let expected_missing_artifacts = rollup_required_u64(&artifact, "missing_artifacts")?;
    let expected_other_failures = rollup_required_u64(&artifact, "other_failures")?;
    let expected_timing_ranges = rollup_required_u64(&artifact, "timing_ranges")?;

    let rollup_host_present = artifact.get("host").is_some_and(Value::is_object);
    let mut valid_bundles = 0;
    let mut drifted_summaries = 0;
    let mut missing_artifacts = 0;
    let mut other_failures = 0;
    let mut timing_ranges = 0;
    let mut bundle_hosts = 0;

    for bundle in bundles {
        let summary = rollup_required_str(bundle, "summary")?;
        let status = rollup_required_str(bundle, "status")?;
        timing_ranges += rollup_required_u64(bundle, "timing_ranges")?;
        if bundle.get("host").is_some_and(Value::is_object) {
            bundle_hosts += 1;
        }
        match status {
            "valid" => valid_bundles += 1,
            "drifted_summary" => drifted_summaries += 1,
            "missing_artifact" => missing_artifacts += 1,
            "other_failure" => other_failures += 1,
            _ => {
                return Err(
                    RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnknownBundleStatus {
                        summary: summary.to_owned(),
                        status: status.to_owned(),
                    },
                );
            }
        }
    }

    let summaries = bundles.len().try_into().map_err(|_| {
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType("bundles")
    })?;
    require_rollup_count("summaries", expected_summaries, summaries)?;
    require_rollup_count("valid_bundles", expected_valid_bundles, valid_bundles)?;
    require_rollup_count(
        "drifted_summaries",
        expected_drifted_summaries,
        drifted_summaries,
    )?;
    require_rollup_count(
        "missing_artifacts",
        expected_missing_artifacts,
        missing_artifacts,
    )?;
    require_rollup_count("other_failures", expected_other_failures, other_failures)?;
    require_rollup_count("timing_ranges", expected_timing_ranges, timing_ranges)?;

    if drifted_summaries != 0 || missing_artifacts != 0 || other_failures != 0 {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::FailedBundles {
                valid_bundles,
                drifted_summaries,
                missing_artifacts,
                other_failures,
            },
        );
    }

    Ok(RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
        path: path.to_path_buf(),
        schema: schema.to_owned(),
        artifact_bytes,
        artifact_fingerprint,
        summaries,
        valid_bundles,
        drifted_summaries,
        missing_artifacts,
        other_failures,
        timing_ranges,
        bundles: summaries,
        rollup_host_present,
        bundle_hosts,
        bundle_hosts_missing: summaries - bundle_hosts,
    })
}

/// Formats a compact JSON line for a successful collection summary rollup check.
///
/// # Errors
///
/// Returns an error when the report cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let line = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA,
        "path": check.path.display().to_string(),
        "rollup_schema": check.schema.as_str(),
        "artifact_bytes": check.artifact_bytes,
        "artifact_fingerprint": check.artifact_fingerprint.as_str(),
        "summaries": check.summaries,
        "valid_bundles": check.valid_bundles,
        "drifted_summaries": check.drifted_summaries,
        "missing_artifacts": check.missing_artifacts,
        "other_failures": check.other_failures,
        "timing_ranges": check.timing_ranges,
        "bundles": check.bundles,
        "rollup_host_present": check.rollup_host_present,
        "bundle_hosts": check.bundle_hosts,
        "bundle_hosts_missing": check.bundle_hosts_missing,
        "status_valid_bundles": check.valid_bundles,
        "status_drifted_summaries": check.drifted_summaries,
        "status_missing_artifacts": check.missing_artifacts,
        "status_other_failures": check.other_failures,
        "artifact": {
            "path": check.path.display().to_string(),
            "rollup_schema": check.schema.as_str(),
            "bytes": check.artifact_bytes,
            "fingerprint": check.artifact_fingerprint.as_str(),
        },
        "counts": {
            "summaries": check.summaries,
            "valid_bundles": check.valid_bundles,
            "timing_ranges": check.timing_ranges,
            "bundles": check.bundles,
        },
        "host_coverage": {
            "rollup_host_present": check.rollup_host_present,
            "bundle_hosts": check.bundle_hosts,
            "bundle_hosts_missing": check.bundle_hosts_missing,
        },
        "status_coverage": {
            "valid_bundles": check.valid_bundles,
            "drifted_summaries": check.drifted_summaries,
            "missing_artifacts": check.missing_artifacts,
            "other_failures": check.other_failures,
        },
    });
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted by a successful collection summary rollup check.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent flat and grouped fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_json_schema(&value)?;
    let check = parse_rollup_check_json_flat_fields(&value)?;
    require_rollup_check_json_top_level_status(&value, &check)?;
    require_rollup_check_json_groups(&value, &check)?;
    Ok(check)
}

/// Summarizes rollup check JSON records found in a saved log.
///
/// # Errors
///
/// Returns an error when no JSON records are found, a record cannot be parsed,
/// or aggregate counters overflow.
pub fn summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let mut summary = RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary::default();
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let check =
                parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                    line,
                )?;
            add_rollup_check_to_log_summary(&mut summary, &check)?;
        }
    }
    if summary.records == 0 {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_json_line",
            ),
        );
    }
    Ok(summary)
}

/// Formats a compact JSON line for a rollup check saved-log summary.
///
/// # Errors
///
/// Returns an error when the summary cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let mut line = rollup_check_log_summary_json_value(summary);
    line["schema"] =
        json!(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA);
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a rollup check saved-log summary.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent flat and grouped fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_json_schema(&value)?;
    let summary = parse_rollup_check_log_summary_json_flat_fields(&value)?;
    require_rollup_check_log_summary_json_groups(&value, &summary)?;
    Ok(summary)
}

/// Parses a compact JSON summary line from a saved rollup check summary log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the summary JSON
/// line is malformed, or no summary JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA)
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_json_line",
        ),
    )
}

/// Verifies an archived rollup check summary JSON line against source records.
///
/// The source log is summarized from rollup-check JSON records, then compared
/// with the archived summary JSON line found in `summary_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed or any summary counter
/// differs between the recomputed source-log summary and the archived summary.
pub fn verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
    source_input: &str,
    summary_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let report =
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
            source_input,
            summary_input,
        )?;
    require_rollup_check_log_summary_json_match(&report)?;
    Ok(report.actual)
}

/// Builds a verification report for an archived summary JSON line.
///
/// The source log is summarized from rollup-check JSON records, then compared
/// with the archived summary JSON line found in `summary_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed.
pub fn check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
    source_input: &str,
    summary_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let expected =
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
            source_input,
        )?;
    let actual =
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
            summary_input,
        )?;
    let drift = first_rollup_check_log_summary_drift(&expected, &actual);
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Formats a compact JSON line for a saved-log summary verification report.
///
/// # Errors
///
/// Returns an error when the report cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let drift = report.drift.as_ref().map_or(Value::Null, |drift| {
        json!({
            "field": drift.field,
            "expected": drift.expected,
            "actual": drift.actual,
        })
    });
    let line = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA,
        "status": report.status_str(),
        "matched": report.is_matched(),
        "expected": rollup_check_log_summary_json_value(&report.expected),
        "actual": rollup_check_log_summary_json_value(&report.actual),
        "drift": drift,
    });
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a saved-log summary verification report.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent status, summary, or drift fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_json_schema(&value)?;
    let expected =
        parse_rollup_check_log_summary_json_value(rollup_required_object(&value, "expected")?)?;
    let actual =
        parse_rollup_check_log_summary_json_value(rollup_required_object(&value, "actual")?)?;
    let computed_drift = first_rollup_check_log_summary_drift(&expected, &actual);
    let drift = parse_rollup_check_log_summary_verification_json_drift(&value)?;
    require_rollup_check_log_summary_verification_json_status(&value, computed_drift.as_ref())?;
    require_rollup_check_log_summary_verification_json_drift_matches(
        computed_drift.as_ref(),
        drift.as_ref(),
    )?;
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Summarizes saved-log summary verification verdict JSON records found in a log.
///
/// # Errors
///
/// Returns an error when no verdict JSON records are found, a record cannot be
/// parsed, or aggregate counters overflow.
pub fn summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let mut rollup =
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup::default(
        );
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA,
                )
            {
                let report =
                    parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(line)?;
                add_rollup_check_log_summary_verification_to_rollup(&mut rollup, &report)?;
            }
        }
    }
    if rollup.records == 0 {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_json_line",
            ),
        );
    }
    Ok(rollup)
}

/// Formats a compact JSON line for a saved-log summary verification rollup.
///
/// # Errors
///
/// Returns an error when the rollup cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let mut line = rollup_check_log_summary_verification_rollup_json_value(rollup);
    line["schema"] =
        json!(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA);
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a saved-log summary verification rollup.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent flat and grouped fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_json_schema(&value)?;
    let rollup = parse_rollup_check_log_summary_verification_rollup_json_flat_fields(&value)?;
    require_rollup_check_log_summary_verification_rollup_json_groups(&value, &rollup)?;
    Ok(rollup)
}

/// Parses a compact JSON rollup line from a saved summary verification rollup log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the rollup JSON
/// line is malformed, or no rollup JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_json_line",
        ),
    )
}

/// Verifies an archived verdict rollup JSON line against source verdict records.
///
/// The source log is summarized from saved summary-verification verdict JSON
/// records, then compared with the archived verdict rollup JSON line found in
/// `rollup_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed or any verdict rollup
/// counter differs between the recomputed source-log rollup and the archived
/// verdict rollup.
pub fn verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
    source_input: &str,
    rollup_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let report =
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
            source_input,
            rollup_input,
        )?;
    require_rollup_check_log_summary_verification_rollup_match(&report)?;
    Ok(report.actual)
}

/// Builds a verification report for an archived verdict rollup JSON line.
///
/// The source log is summarized from saved summary-verification verdict JSON
/// records, then compared with the archived verdict rollup JSON line found in
/// `rollup_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed.
pub fn check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
    source_input: &str,
    rollup_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let expected =
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
            source_input,
        )?;
    let actual =
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
            rollup_input,
        )?;
    let drift = first_rollup_check_log_summary_verification_rollup_drift(&expected, &actual);
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Formats a compact JSON line for a saved verdict rollup verification report.
///
/// # Errors
///
/// Returns an error when the report cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let drift = report.drift.as_ref().map_or(Value::Null, |drift| {
        json!({
            "field": drift.field,
            "expected": drift.expected,
            "actual": drift.actual,
        })
    });
    let line = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
        "status": report.status_str(),
        "matched": report.is_matched(),
        "expected": rollup_check_log_summary_verification_rollup_json_value(&report.expected),
        "actual": rollup_check_log_summary_verification_rollup_json_value(&report.actual),
        "drift": drift,
    });
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a saved verdict rollup verification report.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent status, rollup, or drift fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_verification_json_schema(&value)?;
    let expected = parse_rollup_check_log_summary_verification_rollup_json_value(
        rollup_required_object(&value, "expected")?,
    )?;
    let actual = parse_rollup_check_log_summary_verification_rollup_json_value(
        rollup_required_object(&value, "actual")?,
    )?;
    let computed_drift =
        first_rollup_check_log_summary_verification_rollup_drift(&expected, &actual);
    let drift = parse_rollup_check_log_summary_verification_rollup_verification_json_drift(&value)?;
    require_rollup_check_log_summary_verification_rollup_verification_json_status(
        &value,
        computed_drift.as_ref(),
    )?;
    require_rollup_check_log_summary_verification_rollup_verification_json_drift_matches(
        computed_drift.as_ref(),
        drift.as_ref(),
    )?;
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Parses a compact JSON verification line from a saved verdict rollup verification log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the verification
/// JSON line is malformed, or no verification JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_verification_json_line",
        ),
    )
}

/// Summarizes saved verdict rollup verification JSON records found in a log.
///
/// # Errors
///
/// Returns an error when no verifier JSON records are found, a record cannot
/// be parsed, or aggregate counters overflow.
pub fn summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let mut summary =
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary::default();
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
                )
            {
                let report =
                    parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(line)?;
                add_rollup_check_log_summary_verification_rollup_verification_to_summary(
                    &mut summary,
                    &report,
                )?;
            }
        }
    }
    if summary.records == 0 {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_rollup_verification_json_line",
            ),
        );
    }
    Ok(summary)
}

/// Formats a compact JSON line for a saved verdict rollup verification summary.
///
/// # Errors
///
/// Returns an error when the summary cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let mut line =
        rollup_check_log_summary_verification_rollup_verification_summary_json_value(summary);
    line["schema"] =
        json!(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA);
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Formats a compact JSON line for a saved verifier summary verification report.
///
/// # Errors
///
/// Returns an error when the report cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let drift = report.drift.as_ref().map_or(Value::Null, |drift| {
        json!({
            "field": drift.field,
            "expected": drift.expected,
            "actual": drift.actual,
        })
    });
    let line = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA,
        "status": report.status_str(),
        "matched": report.is_matched(),
        "expected": rollup_check_log_summary_verification_rollup_verification_summary_json_value(&report.expected),
        "actual": rollup_check_log_summary_verification_rollup_verification_summary_json_value(&report.actual),
        "drift": drift,
    });
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a saved verifier summary verification report.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent status, summary, or drift fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_schema(&value)?;
    let expected =
        parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
            rollup_required_object(&value, "expected")?,
        )?;
    let actual =
        parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
            rollup_required_object(&value, "actual")?,
        )?;
    let computed_drift =
        first_rollup_check_log_summary_verification_rollup_verification_summary_drift(
            &expected, &actual,
        );
    let drift =
        parse_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift(&value)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_status(
        &value,
        computed_drift.as_ref(),
    )?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift_matches(
        computed_drift.as_ref(),
        drift.as_ref(),
    )?;
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Parses a compact JSON verification line from a saved verifier summary verification log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the verification
/// JSON line is malformed, or no verification JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line",
        ),
    )
}

/// Summarizes saved verifier-summary verification JSON records found in a log.
///
/// # Errors
///
/// Returns an error when no verifier-summary verification JSON records are
/// found, a record cannot be parsed, or aggregate counters overflow.
pub fn summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let mut rollup =
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup::default();
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA,
                )
            {
                let report =
                    parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(line)?;
                add_rollup_check_log_summary_verification_rollup_verification_summary_verification_to_rollup(
                    &mut rollup,
                    &report,
                )?;
            }
        }
    }
    if rollup.summary.records == 0 {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line",
            ),
        );
    }
    Ok(rollup)
}

/// Formats a compact JSON line for a verifier-summary verification rollup.
///
/// # Errors
///
/// Returns an error when the rollup cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let mut line = rollup_check_log_summary_verification_rollup_verification_summary_json_value(
        &rollup.summary,
    );
    line["schema"] =
        json!(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA);
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a verifier-summary verification rollup.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent flat and grouped fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_schema(&value)?;
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup {
            summary: parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
                &value,
            )?,
        },
    )
}

/// Parses a compact JSON rollup line from a saved verifier-summary verification rollup log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the rollup JSON
/// line is malformed, or no rollup JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line",
        ),
    )
}

/// Verifies an archived verifier-summary verification rollup JSON line against source records.
///
/// The source log is summarized from saved verifier-summary verification JSON
/// records, then compared with the archived rollup JSON line found in
/// `rollup_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed or any rollup counter
/// differs between the recomputed source-log rollup and the archived rollup.
pub fn verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
    source_input: &str,
    rollup_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let report =
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
            source_input,
            rollup_input,
        )?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_match(
        &report,
    )?;
    Ok(report.actual)
}

/// Builds a verification report for an archived verifier-summary verification rollup JSON line.
///
/// The source log is summarized from saved verifier-summary verification JSON
/// records, then compared with the archived rollup JSON line found in
/// `rollup_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed.
pub fn check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
    source_input: &str,
    rollup_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let expected =
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
            source_input,
        )?;
    let actual =
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
            rollup_input,
        )?;
    let drift = first_rollup_check_log_summary_verification_rollup_verification_summary_drift(
        &expected.summary,
        &actual.summary,
    );
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Formats a compact JSON line for a verifier-summary verification rollup verification report.
///
/// # Errors
///
/// Returns an error when the report cannot be serialized as JSON.
pub fn format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification,
) -> Result<String, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let drift = report.drift.as_ref().map_or(Value::Null, |drift| {
        json!({
            "field": drift.field,
            "expected": drift.expected,
            "actual": drift.actual,
        })
    });
    let line = json!({
        "schema": REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
        "status": report.status_str(),
        "matched": report.is_matched(),
        "expected": rollup_check_log_summary_verification_rollup_verification_summary_json_value(&report.expected.summary),
        "actual": rollup_check_log_summary_verification_rollup_verification_summary_json_value(&report.actual.summary),
        "drift": drift,
    });
    serde_json::to_string(&line)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Serialize)
}

/// Parses a compact JSON line emitted for a verifier-summary verification rollup verification report.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent status, rollup, or drift fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_schema(&value)?;
    let expected =
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup {
            summary: parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
                rollup_required_object(&value, "expected")?,
            )?,
        };
    let actual =
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup {
            summary: parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
                rollup_required_object(&value, "actual")?,
            )?,
        };
    let computed_drift =
        first_rollup_check_log_summary_verification_rollup_verification_summary_drift(
            &expected.summary,
            &actual.summary,
        );
    let drift =
        parse_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift(&value)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_status(
        &value,
        computed_drift.as_ref(),
    )?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift_matches(
        computed_drift.as_ref(),
        drift.as_ref(),
    )?;
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification {
            expected,
            actual,
            drift,
        },
    )
}

/// Parses a compact JSON verification line from a verifier-summary verification rollup verification log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the verification
/// JSON line is malformed, or no verification JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line",
        ),
    )
}

/// Parses a compact JSON line emitted for a saved verdict rollup verification summary.
///
/// # Errors
///
/// Returns an error when the line is malformed, has an unexpected schema, or
/// has inconsistent flat and grouped fields.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let value = serde_json::from_str::<Value>(input)
        .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_json_schema(&value)?;
    parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(&value)
}

/// Parses a compact JSON summary line from a saved verdict rollup verification summary log.
///
/// # Errors
///
/// Returns an error when a candidate JSON line is malformed, the summary JSON
/// line is malformed, or no summary JSON line is found.
pub fn parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
    input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            let value = serde_json::from_str::<Value>(line)
                .map_err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::Json)?;
            if value.get("schema").and_then(Value::as_str)
                == Some(
                    REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA,
                )
            {
                return parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(line);
            }
        }
    }
    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
            "rollup_check_log_summary_verification_rollup_verification_summary_json_line",
        ),
    )
}

/// Verifies an archived verifier-summary JSON line against source verifier records.
///
/// The source log is summarized from saved verifier JSON records, then compared
/// with the archived verifier-summary JSON line found in `summary_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed or any verifier-summary
/// counter differs between the recomputed source-log summary and the archived
/// verifier summary.
pub fn verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
    source_input: &str,
    summary_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let report =
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
            source_input,
            summary_input,
        )?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_match(&report)?;
    Ok(report.actual)
}

/// Builds a verification report for an archived verifier-summary JSON line.
///
/// The source log is summarized from saved verifier JSON records, then compared
/// with the archived verifier-summary JSON line found in `summary_input`.
///
/// # Errors
///
/// Returns an error when either input is malformed.
pub fn check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
    source_input: &str,
    summary_input: &str,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let expected =
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
            source_input,
        )?;
    let actual =
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
            summary_input,
        )?;
    let drift = first_rollup_check_log_summary_verification_rollup_verification_summary_drift(
        &expected, &actual,
    );
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification {
            expected,
            actual,
            drift,
        },
    )
}

fn require_rollup_check_log_summary_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn parse_rollup_check_log_summary_verification_rollup_json_value(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let rollup = parse_rollup_check_log_summary_verification_rollup_json_flat_fields(value)?;
    require_rollup_check_log_summary_verification_rollup_json_groups(value, &rollup)?;
    Ok(rollup)
}

fn parse_rollup_check_log_summary_verification_rollup_verification_summary_json_value(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let summary =
        parse_rollup_check_log_summary_verification_rollup_verification_summary_json_flat_fields(
            value,
        )?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_json_groups(
        value, &summary,
    )?;
    Ok(summary)
}

fn parse_rollup_check_log_summary_verification_rollup_json_flat_fields(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
            records: rollup_required_u64(value, "records")?,
            matched: rollup_required_u64(value, "matched")?,
            drifted: rollup_required_u64(value, "drifted")?,
            drift_records: rollup_required_u64(value, "drift_records")?,
            drift_rollup_hosts_present: rollup_required_u64(value, "drift_rollup_hosts_present")?,
            drift_rollup_hosts_missing: rollup_required_u64(value, "drift_rollup_hosts_missing")?,
            drift_bundle_hosts: rollup_required_u64(value, "drift_bundle_hosts")?,
            drift_bundle_hosts_missing: rollup_required_u64(value, "drift_bundle_hosts_missing")?,
            drift_status_valid_bundles: rollup_required_u64(value, "drift_status_valid_bundles")?,
            drift_status_drifted_summaries: rollup_required_u64(
                value,
                "drift_status_drifted_summaries",
            )?,
            drift_status_missing_artifacts: rollup_required_u64(
                value,
                "drift_status_missing_artifacts",
            )?,
            drift_status_other_failures: rollup_required_u64(value, "drift_status_other_failures")?,
        },
    )
}

fn require_rollup_check_log_summary_verification_rollup_json_groups(
    value: &Value,
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let status_coverage = rollup_required_object(value, "status_coverage")?;
    require_rollup_count(
        "status_coverage.matched",
        rollup.matched,
        rollup_required_u64(status_coverage, "matched")?,
    )?;
    require_rollup_count(
        "status_coverage.drifted",
        rollup.drifted,
        rollup_required_u64(status_coverage, "drifted")?,
    )?;
    let drift_fields = rollup_required_object(value, "drift_fields")?;
    require_rollup_count(
        "drift_fields.records",
        rollup.drift_records,
        rollup_required_u64(drift_fields, "records")?,
    )?;
    require_rollup_count(
        "drift_fields.rollup_hosts_present",
        rollup.drift_rollup_hosts_present,
        rollup_required_u64(drift_fields, "rollup_hosts_present")?,
    )?;
    require_rollup_count(
        "drift_fields.rollup_hosts_missing",
        rollup.drift_rollup_hosts_missing,
        rollup_required_u64(drift_fields, "rollup_hosts_missing")?,
    )?;
    require_rollup_count(
        "drift_fields.bundle_hosts",
        rollup.drift_bundle_hosts,
        rollup_required_u64(drift_fields, "bundle_hosts")?,
    )?;
    require_rollup_count(
        "drift_fields.bundle_hosts_missing",
        rollup.drift_bundle_hosts_missing,
        rollup_required_u64(drift_fields, "bundle_hosts_missing")?,
    )?;
    require_rollup_count(
        "drift_fields.status_valid_bundles",
        rollup.drift_status_valid_bundles,
        rollup_required_u64(drift_fields, "status_valid_bundles")?,
    )?;
    require_rollup_count(
        "drift_fields.status_drifted_summaries",
        rollup.drift_status_drifted_summaries,
        rollup_required_u64(drift_fields, "status_drifted_summaries")?,
    )?;
    require_rollup_count(
        "drift_fields.status_missing_artifacts",
        rollup.drift_status_missing_artifacts,
        rollup_required_u64(drift_fields, "status_missing_artifacts")?,
    )?;
    require_rollup_count(
        "drift_fields.status_other_failures",
        rollup.drift_status_other_failures,
        rollup_required_u64(drift_fields, "status_other_failures")?,
    )?;
    Ok(())
}

fn parse_rollup_check_log_summary_verification_rollup_verification_summary_json_flat_fields(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary {
            records: rollup_required_u64(value, "records")?,
            matched: rollup_required_u64(value, "matched")?,
            drifted: rollup_required_u64(value, "drifted")?,
            drift_records: rollup_required_u64(value, "drift_records")?,
            drift_matched: rollup_required_u64(value, "drift_matched")?,
            drift_drifted: rollup_required_u64(value, "drift_drifted")?,
            drift_drift_records: rollup_required_u64(value, "drift_drift_records")?,
            drift_drift_rollup_hosts_present: rollup_required_u64(
                value,
                "drift_drift_rollup_hosts_present",
            )?,
            drift_drift_rollup_hosts_missing: rollup_required_u64(
                value,
                "drift_drift_rollup_hosts_missing",
            )?,
            drift_drift_bundle_hosts: rollup_required_u64(
                value,
                "drift_drift_bundle_hosts",
            )?,
            drift_drift_bundle_hosts_missing: rollup_required_u64(
                value,
                "drift_drift_bundle_hosts_missing",
            )?,
            drift_drift_status_valid_bundles: rollup_required_u64(
                value,
                "drift_drift_status_valid_bundles",
            )?,
            drift_drift_status_drifted_summaries: rollup_required_u64(
                value,
                "drift_drift_status_drifted_summaries",
            )?,
            drift_drift_status_missing_artifacts: rollup_required_u64(
                value,
                "drift_drift_status_missing_artifacts",
            )?,
            drift_drift_status_other_failures: rollup_required_u64(
                value,
                "drift_drift_status_other_failures",
            )?,
        },
    )
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_json_groups(
    value: &Value,
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let status_coverage = rollup_required_object(value, "status_coverage")?;
    require_rollup_count(
        "status_coverage.matched",
        summary.matched,
        rollup_required_u64(status_coverage, "matched")?,
    )?;
    require_rollup_count(
        "status_coverage.drifted",
        summary.drifted,
        rollup_required_u64(status_coverage, "drifted")?,
    )?;

    let drift_fields = rollup_required_object(value, "drift_fields")?;
    require_rollup_check_log_summary_verification_rollup_verification_summary_drift_fields(
        drift_fields,
        summary,
    )
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_drift_fields(
    drift_fields: &Value,
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let expected_fields = [
        ("records", summary.drift_records),
        ("matched", summary.drift_matched),
        ("drifted", summary.drift_drifted),
        ("drift_records", summary.drift_drift_records),
        (
            "drift_rollup_hosts_present",
            summary.drift_drift_rollup_hosts_present,
        ),
        (
            "drift_rollup_hosts_missing",
            summary.drift_drift_rollup_hosts_missing,
        ),
        ("drift_bundle_hosts", summary.drift_drift_bundle_hosts),
        (
            "drift_bundle_hosts_missing",
            summary.drift_drift_bundle_hosts_missing,
        ),
        (
            "drift_status_valid_bundles",
            summary.drift_drift_status_valid_bundles,
        ),
        (
            "drift_status_drifted_summaries",
            summary.drift_drift_status_drifted_summaries,
        ),
        (
            "drift_status_missing_artifacts",
            summary.drift_drift_status_missing_artifacts,
        ),
        (
            "drift_status_other_failures",
            summary.drift_drift_status_other_failures,
        ),
    ];
    for (field, expected) in expected_fields {
        require_rollup_count(
            rollup_check_log_summary_verification_rollup_verification_summary_drift_group_field(
                field,
            ),
            expected,
            rollup_required_u64(drift_fields, field)?,
        )?;
    }
    Ok(())
}

fn rollup_check_log_summary_verification_rollup_verification_summary_drift_group_field(
    field: &'static str,
) -> &'static str {
    match field {
        "records" => "drift_fields.records",
        "matched" => "drift_fields.matched",
        "drifted" => "drift_fields.drifted",
        "drift_records" => "drift_fields.drift_records",
        "drift_rollup_hosts_present" => "drift_fields.drift_rollup_hosts_present",
        "drift_rollup_hosts_missing" => "drift_fields.drift_rollup_hosts_missing",
        "drift_bundle_hosts" => "drift_fields.drift_bundle_hosts",
        "drift_bundle_hosts_missing" => "drift_fields.drift_bundle_hosts_missing",
        "drift_status_valid_bundles" => "drift_fields.drift_status_valid_bundles",
        "drift_status_drifted_summaries" => "drift_fields.drift_status_drifted_summaries",
        "drift_status_missing_artifacts" => "drift_fields.drift_status_missing_artifacts",
        "drift_status_other_failures" => "drift_fields.drift_status_other_failures",
        _ => "drift_fields.unknown",
    }
}

fn parse_rollup_check_log_summary_verification_rollup_verification_json_drift(
    value: &Value,
) -> Result<
    Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift>,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let drift = value
        .as_object()
        .and_then(|object| object.get("drift"))
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("drift"))?;
    if drift.is_null() {
        return Ok(None);
    }
    let field = rollup_check_log_summary_verification_rollup_drift_field(rollup_required_str(
        drift, "field",
    )?)?;
    Ok(Some(rollup_check_log_summary_verification_rollup_drift(
        field,
        rollup_required_u64(drift, "expected")?,
        rollup_required_u64(drift, "actual")?,
    )))
}

fn require_rollup_check_log_summary_verification_rollup_verification_json_status(
    value: &Value,
    drift: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift,
    >,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let expected_status = if drift.is_some() {
        "drifted"
    } else {
        "matched"
    };
    require_rollup_json_field(
        "status",
        expected_status,
        rollup_required_str(value, "status")?,
    )?;
    let matched = rollup_required_bool(value, "matched")?;
    if matched != drift.is_none() {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "matched",
                expected: drift.is_none().to_string(),
                actual: matched.to_string(),
            },
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_json_drift_matches(
    expected: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift,
    >,
    actual: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift,
    >,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match (expected, actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual))
            if expected.field == actual.field
                && expected.expected == actual.expected
                && expected.actual == actual.actual =>
        {
            Ok(())
        }
        (expected, actual) => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                expected: rollup_check_log_summary_verification_rollup_drift_label(expected),
                actual: rollup_check_log_summary_verification_rollup_drift_label(actual),
            },
        ),
    }
}

fn rollup_check_log_summary_verification_rollup_drift_field(
    field: &str,
) -> Result<&'static str, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match field {
        "records" => Ok("records"),
        "matched" => Ok("matched"),
        "drifted" => Ok("drifted"),
        "drift_records" => Ok("drift_records"),
        "drift_rollup_hosts_present" => Ok("drift_rollup_hosts_present"),
        "drift_rollup_hosts_missing" => Ok("drift_rollup_hosts_missing"),
        "drift_bundle_hosts" => Ok("drift_bundle_hosts"),
        "drift_bundle_hosts_missing" => Ok("drift_bundle_hosts_missing"),
        "drift_status_valid_bundles" => Ok("drift_status_valid_bundles"),
        "drift_status_drifted_summaries" => Ok("drift_status_drifted_summaries"),
        "drift_status_missing_artifacts" => Ok("drift_status_missing_artifacts"),
        "drift_status_other_failures" => Ok("drift_status_other_failures"),
        _ => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift.field",
                expected: "known_verdict_rollup_counter".to_owned(),
                actual: field.to_owned(),
            },
        ),
    }
}

fn rollup_check_log_summary_verification_rollup_drift_label(
    drift: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift,
    >,
) -> String {
    drift.map_or_else(
        || "none".to_owned(),
        |drift| format!("{}:{}:{}", drift.field, drift.expected, drift.actual),
    )
}

fn parse_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift(
    value: &Value,
) -> Result<
    Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift>,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
>{
    let drift = value
        .as_object()
        .and_then(|object| object.get("drift"))
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("drift"))?;
    if drift.is_null() {
        return Ok(None);
    }
    let field = rollup_check_log_summary_verification_rollup_verification_summary_drift_field(
        rollup_required_str(drift, "field")?,
    )?;
    Ok(Some(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift {
            field,
            expected: rollup_required_u64(drift, "expected")?,
            actual: rollup_required_u64(drift, "actual")?,
        },
    ))
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_status(
    value: &Value,
    drift: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let expected_status = if drift.is_some() {
        "drifted"
    } else {
        "matched"
    };
    require_rollup_json_field(
        "status",
        expected_status,
        rollup_required_str(value, "status")?,
    )?;
    let matched = rollup_required_bool(value, "matched")?;
    if matched != drift.is_none() {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "matched",
                expected: drift.is_none().to_string(),
                actual: matched.to_string(),
            },
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_drift_matches(
    expected: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
    actual: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match (expected, actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual))
            if expected.field == actual.field
                && expected.expected == actual.expected
                && expected.actual == actual.actual =>
        {
            Ok(())
        }
        (expected, actual) => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                expected:
                    rollup_check_log_summary_verification_rollup_verification_summary_drift_label(
                        expected,
                    ),
                actual:
                    rollup_check_log_summary_verification_rollup_verification_summary_drift_label(
                        actual,
                    ),
            },
        ),
    }
}

fn rollup_check_log_summary_verification_rollup_verification_summary_drift_field(
    field: &str,
) -> Result<&'static str, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match field {
        "records" => Ok("records"),
        "matched" => Ok("matched"),
        "drifted" => Ok("drifted"),
        "drift_records" => Ok("drift_records"),
        "drift_matched" => Ok("drift_matched"),
        "drift_drifted" => Ok("drift_drifted"),
        "drift_drift_records" => Ok("drift_drift_records"),
        "drift_drift_rollup_hosts_present" => Ok("drift_drift_rollup_hosts_present"),
        "drift_drift_rollup_hosts_missing" => Ok("drift_drift_rollup_hosts_missing"),
        "drift_drift_bundle_hosts" => Ok("drift_drift_bundle_hosts"),
        "drift_drift_bundle_hosts_missing" => Ok("drift_drift_bundle_hosts_missing"),
        "drift_drift_status_valid_bundles" => Ok("drift_drift_status_valid_bundles"),
        "drift_drift_status_drifted_summaries" => Ok("drift_drift_status_drifted_summaries"),
        "drift_drift_status_missing_artifacts" => Ok("drift_drift_status_missing_artifacts"),
        "drift_drift_status_other_failures" => Ok("drift_drift_status_other_failures"),
        _ => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift.field",
                expected: "known_verifier_summary_counter".to_owned(),
                actual: field.to_owned(),
            },
        ),
    }
}

fn rollup_check_log_summary_verification_rollup_verification_summary_drift_label(
    drift: Option<
        &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift,
    >,
) -> String {
    drift.map_or_else(
        || "none".to_owned(),
        |drift| format!("{}:{}:{}", drift.field, drift.expected, drift.actual),
    )
}

fn require_rollup_check_log_summary_verification_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema
        != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn parse_rollup_check_log_summary_json_value(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let summary = parse_rollup_check_log_summary_json_flat_fields(value)?;
    require_rollup_check_log_summary_json_groups(value, &summary)?;
    Ok(summary)
}

fn parse_rollup_check_log_summary_json_flat_fields(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    Ok(
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary {
            records: rollup_required_u64(value, "records")?,
            rollup_hosts_present: rollup_required_u64(value, "rollup_hosts_present")?,
            rollup_hosts_missing: rollup_required_u64(value, "rollup_hosts_missing")?,
            bundle_hosts: rollup_required_u64(value, "bundle_hosts")?,
            bundle_hosts_missing: rollup_required_u64(value, "bundle_hosts_missing")?,
            status_valid_bundles: rollup_required_u64(value, "status_valid_bundles")?,
            status_drifted_summaries: rollup_required_u64(value, "status_drifted_summaries")?,
            status_missing_artifacts: rollup_required_u64(value, "status_missing_artifacts")?,
            status_other_failures: rollup_required_u64(value, "status_other_failures")?,
        },
    )
}

fn parse_rollup_check_log_summary_verification_json_drift(
    value: &Value,
) -> Result<
    Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let drift = value
        .as_object()
        .and_then(|object| object.get("drift"))
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("drift"))?;
    if drift.is_null() {
        return Ok(None);
    }
    let field = rollup_required_str(drift, "field")?;
    let field = rollup_check_log_summary_drift_field(field)?;
    Ok(Some(rollup_check_log_summary_drift(
        field,
        rollup_required_u64(drift, "expected")?,
        rollup_required_u64(drift, "actual")?,
    )))
}

fn require_rollup_check_log_summary_verification_json_status(
    value: &Value,
    drift: Option<&RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let expected_status = if drift.is_some() {
        "drifted"
    } else {
        "matched"
    };
    require_rollup_json_field(
        "status",
        expected_status,
        rollup_required_str(value, "status")?,
    )?;
    let matched = rollup_required_bool(value, "matched")?;
    if matched != drift.is_none() {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "matched",
                expected: drift.is_none().to_string(),
                actual: matched.to_string(),
            },
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_json_drift_matches(
    expected: Option<&RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
    actual: Option<&RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match (expected, actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual))
            if expected.field == actual.field
                && expected.expected == actual.expected
                && expected.actual == actual.actual =>
        {
            Ok(())
        }
        (expected, actual) => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                expected: rollup_check_log_summary_drift_label(expected),
                actual: rollup_check_log_summary_drift_label(actual),
            },
        ),
    }
}

fn rollup_check_log_summary_drift_field(
    field: &str,
) -> Result<&'static str, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match field {
        "records" => Ok("records"),
        "rollup_hosts_present" => Ok("rollup_hosts_present"),
        "rollup_hosts_missing" => Ok("rollup_hosts_missing"),
        "bundle_hosts" => Ok("bundle_hosts"),
        "bundle_hosts_missing" => Ok("bundle_hosts_missing"),
        "status_valid_bundles" => Ok("status_valid_bundles"),
        "status_drifted_summaries" => Ok("status_drifted_summaries"),
        "status_missing_artifacts" => Ok("status_missing_artifacts"),
        "status_other_failures" => Ok("status_other_failures"),
        _ => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift.field",
                expected: "known_summary_counter".to_owned(),
                actual: field.to_owned(),
            },
        ),
    }
}

fn rollup_check_log_summary_drift_label(
    drift: Option<&RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift>,
) -> String {
    drift.map_or_else(
        || "none".to_owned(),
        |drift| format!("{}:{}:{}", drift.field, drift.expected, drift.actual),
    )
}

fn require_rollup_check_log_summary_json_groups(
    value: &Value,
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    require_rollup_check_log_summary_json_host_coverage_group(value, summary)?;
    require_rollup_check_log_summary_json_status_coverage_group(value, summary)?;
    Ok(())
}

fn require_rollup_check_log_summary_json_host_coverage_group(
    value: &Value,
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let host_coverage = rollup_required_object(value, "host_coverage")?;
    require_rollup_count(
        "host_coverage.rollup_hosts_present",
        summary.rollup_hosts_present,
        rollup_required_u64(host_coverage, "rollup_hosts_present")?,
    )?;
    require_rollup_count(
        "host_coverage.rollup_hosts_missing",
        summary.rollup_hosts_missing,
        rollup_required_u64(host_coverage, "rollup_hosts_missing")?,
    )?;
    require_rollup_count(
        "host_coverage.bundle_hosts",
        summary.bundle_hosts,
        rollup_required_u64(host_coverage, "bundle_hosts")?,
    )?;
    require_rollup_count(
        "host_coverage.bundle_hosts_missing",
        summary.bundle_hosts_missing,
        rollup_required_u64(host_coverage, "bundle_hosts_missing")?,
    )?;
    Ok(())
}

fn require_rollup_check_log_summary_json_status_coverage_group(
    value: &Value,
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let status_coverage = rollup_required_object(value, "status_coverage")?;
    require_rollup_count(
        "status_coverage.valid_bundles",
        summary.status_valid_bundles,
        rollup_required_u64(status_coverage, "valid_bundles")?,
    )?;
    require_rollup_count(
        "status_coverage.drifted_summaries",
        summary.status_drifted_summaries,
        rollup_required_u64(status_coverage, "drifted_summaries")?,
    )?;
    require_rollup_count(
        "status_coverage.missing_artifacts",
        summary.status_missing_artifacts,
        rollup_required_u64(status_coverage, "missing_artifacts")?,
    )?;
    require_rollup_count(
        "status_coverage.other_failures",
        summary.status_other_failures,
        rollup_required_u64(status_coverage, "other_failures")?,
    )?;
    Ok(())
}

fn require_rollup_check_log_summary_json_match(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if let Some(drift) = &report.drift {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: drift.field,
                expected: drift.expected,
                actual: drift.actual,
            },
        );
    }
    Ok(())
}

fn rollup_check_log_summary_json_value(
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Value {
    json!({
        "records": summary.records,
        "rollup_hosts_present": summary.rollup_hosts_present,
        "rollup_hosts_missing": summary.rollup_hosts_missing,
        "bundle_hosts": summary.bundle_hosts,
        "bundle_hosts_missing": summary.bundle_hosts_missing,
        "status_valid_bundles": summary.status_valid_bundles,
        "status_drifted_summaries": summary.status_drifted_summaries,
        "status_missing_artifacts": summary.status_missing_artifacts,
        "status_other_failures": summary.status_other_failures,
        "host_coverage": {
            "rollup_hosts_present": summary.rollup_hosts_present,
            "rollup_hosts_missing": summary.rollup_hosts_missing,
            "bundle_hosts": summary.bundle_hosts,
            "bundle_hosts_missing": summary.bundle_hosts_missing,
        },
        "status_coverage": {
            "valid_bundles": summary.status_valid_bundles,
            "drifted_summaries": summary.status_drifted_summaries,
            "missing_artifacts": summary.status_missing_artifacts,
            "other_failures": summary.status_other_failures,
        },
    })
}

fn first_rollup_check_log_summary_drift(
    expected: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    actual: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
) -> Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift> {
    if expected.records != actual.records {
        return Some(rollup_check_log_summary_drift(
            "records",
            expected.records,
            actual.records,
        ));
    }
    if expected.rollup_hosts_present != actual.rollup_hosts_present {
        return Some(rollup_check_log_summary_drift(
            "rollup_hosts_present",
            expected.rollup_hosts_present,
            actual.rollup_hosts_present,
        ));
    }
    if expected.rollup_hosts_missing != actual.rollup_hosts_missing {
        return Some(rollup_check_log_summary_drift(
            "rollup_hosts_missing",
            expected.rollup_hosts_missing,
            actual.rollup_hosts_missing,
        ));
    }
    if expected.bundle_hosts != actual.bundle_hosts {
        return Some(rollup_check_log_summary_drift(
            "bundle_hosts",
            expected.bundle_hosts,
            actual.bundle_hosts,
        ));
    }
    if expected.bundle_hosts_missing != actual.bundle_hosts_missing {
        return Some(rollup_check_log_summary_drift(
            "bundle_hosts_missing",
            expected.bundle_hosts_missing,
            actual.bundle_hosts_missing,
        ));
    }
    if expected.status_valid_bundles != actual.status_valid_bundles {
        return Some(rollup_check_log_summary_drift(
            "status_valid_bundles",
            expected.status_valid_bundles,
            actual.status_valid_bundles,
        ));
    }
    if expected.status_drifted_summaries != actual.status_drifted_summaries {
        return Some(rollup_check_log_summary_drift(
            "status_drifted_summaries",
            expected.status_drifted_summaries,
            actual.status_drifted_summaries,
        ));
    }
    if expected.status_missing_artifacts != actual.status_missing_artifacts {
        return Some(rollup_check_log_summary_drift(
            "status_missing_artifacts",
            expected.status_missing_artifacts,
            actual.status_missing_artifacts,
        ));
    }
    if expected.status_other_failures != actual.status_other_failures {
        return Some(rollup_check_log_summary_drift(
            "status_other_failures",
            expected.status_other_failures,
            actual.status_other_failures,
        ));
    }
    None
}

fn rollup_check_log_summary_drift(
    field: &'static str,
    expected: u64,
    actual: u64,
) -> RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift {
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift {
        field,
        expected,
        actual,
    }
}

fn add_rollup_check_log_summary_verification_to_rollup(
    rollup: &mut RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    rollup.records = checked_add_rollup_check_log_summary_count(rollup.records, 1, "records")?;
    if let Some(drift) = &report.drift {
        rollup.drifted = checked_add_rollup_check_log_summary_count(rollup.drifted, 1, "drifted")?;
        match drift.field {
            "records" => {
                rollup.drift_records = checked_add_rollup_check_log_summary_count(
                    rollup.drift_records,
                    1,
                    "drift_records",
                )?;
            }
            "rollup_hosts_present" => {
                rollup.drift_rollup_hosts_present = checked_add_rollup_check_log_summary_count(
                    rollup.drift_rollup_hosts_present,
                    1,
                    "drift_rollup_hosts_present",
                )?;
            }
            "rollup_hosts_missing" => {
                rollup.drift_rollup_hosts_missing = checked_add_rollup_check_log_summary_count(
                    rollup.drift_rollup_hosts_missing,
                    1,
                    "drift_rollup_hosts_missing",
                )?;
            }
            "bundle_hosts" => {
                rollup.drift_bundle_hosts = checked_add_rollup_check_log_summary_count(
                    rollup.drift_bundle_hosts,
                    1,
                    "drift_bundle_hosts",
                )?;
            }
            "bundle_hosts_missing" => {
                rollup.drift_bundle_hosts_missing = checked_add_rollup_check_log_summary_count(
                    rollup.drift_bundle_hosts_missing,
                    1,
                    "drift_bundle_hosts_missing",
                )?;
            }
            "status_valid_bundles" => {
                rollup.drift_status_valid_bundles = checked_add_rollup_check_log_summary_count(
                    rollup.drift_status_valid_bundles,
                    1,
                    "drift_status_valid_bundles",
                )?;
            }
            "status_drifted_summaries" => {
                rollup.drift_status_drifted_summaries = checked_add_rollup_check_log_summary_count(
                    rollup.drift_status_drifted_summaries,
                    1,
                    "drift_status_drifted_summaries",
                )?;
            }
            "status_missing_artifacts" => {
                rollup.drift_status_missing_artifacts = checked_add_rollup_check_log_summary_count(
                    rollup.drift_status_missing_artifacts,
                    1,
                    "drift_status_missing_artifacts",
                )?;
            }
            "status_other_failures" => {
                rollup.drift_status_other_failures = checked_add_rollup_check_log_summary_count(
                    rollup.drift_status_other_failures,
                    1,
                    "drift_status_other_failures",
                )?;
            }
            _ => {
                return Err(
                    RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                        field: "drift.field",
                        expected: "known_summary_counter".to_owned(),
                        actual: drift.field.to_owned(),
                    },
                );
            }
        }
    } else {
        rollup.matched = checked_add_rollup_check_log_summary_count(rollup.matched, 1, "matched")?;
    }
    Ok(())
}

fn add_rollup_check_log_summary_verification_rollup_verification_to_summary(
    summary: &mut RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    summary.records = checked_add_rollup_check_log_summary_count(summary.records, 1, "records")?;
    if let Some(drift) = &report.drift {
        summary.drifted =
            checked_add_rollup_check_log_summary_count(summary.drifted, 1, "drifted")?;
        let (bucket, field) =
            rollup_check_log_summary_verification_rollup_verification_summary_drift_bucket(
                summary,
                drift.field,
            )?;
        *bucket = checked_add_rollup_check_log_summary_count(*bucket, 1, field)?;
    } else {
        summary.matched =
            checked_add_rollup_check_log_summary_count(summary.matched, 1, "matched")?;
    }
    Ok(())
}

fn add_rollup_check_log_summary_verification_rollup_verification_summary_verification_to_rollup(
    rollup: &mut RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollup,
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let summary = &mut rollup.summary;
    summary.records = checked_add_rollup_check_log_summary_count(summary.records, 1, "records")?;
    if let Some(drift) = &report.drift {
        summary.drifted =
            checked_add_rollup_check_log_summary_count(summary.drifted, 1, "drifted")?;
        let (bucket, field) =
            rollup_check_log_summary_verification_rollup_verification_summary_drift_bucket(
                summary,
                drift.field,
            )?;
        *bucket = checked_add_rollup_check_log_summary_count(*bucket, 1, field)?;
    } else {
        summary.matched =
            checked_add_rollup_check_log_summary_count(summary.matched, 1, "matched")?;
    }
    Ok(())
}

fn rollup_check_log_summary_verification_rollup_verification_summary_drift_bucket<'a>(
    summary: &'a mut RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    field: &'static str,
) -> Result<(&'a mut u64, &'static str), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    match field {
        "records" => Ok((&mut summary.drift_records, "drift_records")),
        "matched" => Ok((&mut summary.drift_matched, "drift_matched")),
        "drifted" => Ok((&mut summary.drift_drifted, "drift_drifted")),
        "drift_records" => Ok((&mut summary.drift_drift_records, "drift_drift_records")),
        "drift_rollup_hosts_present" => Ok((
            &mut summary.drift_drift_rollup_hosts_present,
            "drift_drift_rollup_hosts_present",
        )),
        "drift_rollup_hosts_missing" => Ok((
            &mut summary.drift_drift_rollup_hosts_missing,
            "drift_drift_rollup_hosts_missing",
        )),
        "drift_bundle_hosts" => Ok((
            &mut summary.drift_drift_bundle_hosts,
            "drift_drift_bundle_hosts",
        )),
        "drift_bundle_hosts_missing" => Ok((
            &mut summary.drift_drift_bundle_hosts_missing,
            "drift_drift_bundle_hosts_missing",
        )),
        "drift_status_valid_bundles" => Ok((
            &mut summary.drift_drift_status_valid_bundles,
            "drift_drift_status_valid_bundles",
        )),
        "drift_status_drifted_summaries" => Ok((
            &mut summary.drift_drift_status_drifted_summaries,
            "drift_drift_status_drifted_summaries",
        )),
        "drift_status_missing_artifacts" => Ok((
            &mut summary.drift_drift_status_missing_artifacts,
            "drift_drift_status_missing_artifacts",
        )),
        "drift_status_other_failures" => Ok((
            &mut summary.drift_drift_status_other_failures,
            "drift_drift_status_other_failures",
        )),
        _ => Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift.field",
                expected: "known_verdict_rollup_counter".to_owned(),
                actual: field.to_owned(),
            },
        ),
    }
}

fn require_rollup_check_log_summary_verification_rollup_match(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if let Some(drift) = &report.drift {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: drift.field,
                expected: drift.expected,
                actual: drift.actual,
            },
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_match(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if let Some(drift) = &report.drift {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: drift.field,
                expected: drift.expected,
                actual: drift.actual,
            },
        );
    }
    Ok(())
}

fn require_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_match(
    report: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryVerificationRollupVerification,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if let Some(drift) = &report.drift {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: drift.field,
                expected: drift.expected,
                actual: drift.actual,
            },
        );
    }
    Ok(())
}

fn rollup_check_log_summary_verification_rollup_json_value(
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
) -> Value {
    json!({
        "records": rollup.records,
        "matched": rollup.matched,
        "drifted": rollup.drifted,
        "drift_records": rollup.drift_records,
        "drift_rollup_hosts_present": rollup.drift_rollup_hosts_present,
        "drift_rollup_hosts_missing": rollup.drift_rollup_hosts_missing,
        "drift_bundle_hosts": rollup.drift_bundle_hosts,
        "drift_bundle_hosts_missing": rollup.drift_bundle_hosts_missing,
        "drift_status_valid_bundles": rollup.drift_status_valid_bundles,
        "drift_status_drifted_summaries": rollup.drift_status_drifted_summaries,
        "drift_status_missing_artifacts": rollup.drift_status_missing_artifacts,
        "drift_status_other_failures": rollup.drift_status_other_failures,
        "status_coverage": {
            "matched": rollup.matched,
            "drifted": rollup.drifted,
        },
        "drift_fields": {
            "records": rollup.drift_records,
            "rollup_hosts_present": rollup.drift_rollup_hosts_present,
            "rollup_hosts_missing": rollup.drift_rollup_hosts_missing,
            "bundle_hosts": rollup.drift_bundle_hosts,
            "bundle_hosts_missing": rollup.drift_bundle_hosts_missing,
            "status_valid_bundles": rollup.drift_status_valid_bundles,
            "status_drifted_summaries": rollup.drift_status_drifted_summaries,
            "status_missing_artifacts": rollup.drift_status_missing_artifacts,
            "status_other_failures": rollup.drift_status_other_failures,
        },
    })
}

fn rollup_check_log_summary_verification_rollup_verification_summary_json_value(
    summary: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
) -> Value {
    json!({
        "records": summary.records,
        "matched": summary.matched,
        "drifted": summary.drifted,
        "drift_records": summary.drift_records,
        "drift_matched": summary.drift_matched,
        "drift_drifted": summary.drift_drifted,
        "drift_drift_records": summary.drift_drift_records,
        "drift_drift_rollup_hosts_present": summary.drift_drift_rollup_hosts_present,
        "drift_drift_rollup_hosts_missing": summary.drift_drift_rollup_hosts_missing,
        "drift_drift_bundle_hosts": summary.drift_drift_bundle_hosts,
        "drift_drift_bundle_hosts_missing": summary.drift_drift_bundle_hosts_missing,
        "drift_drift_status_valid_bundles": summary.drift_drift_status_valid_bundles,
        "drift_drift_status_drifted_summaries": summary.drift_drift_status_drifted_summaries,
        "drift_drift_status_missing_artifacts": summary.drift_drift_status_missing_artifacts,
        "drift_drift_status_other_failures": summary.drift_drift_status_other_failures,
        "status_coverage": {
            "matched": summary.matched,
            "drifted": summary.drifted,
        },
        "drift_fields": {
            "records": summary.drift_records,
            "matched": summary.drift_matched,
            "drifted": summary.drift_drifted,
            "drift_records": summary.drift_drift_records,
            "drift_rollup_hosts_present": summary.drift_drift_rollup_hosts_present,
            "drift_rollup_hosts_missing": summary.drift_drift_rollup_hosts_missing,
            "drift_bundle_hosts": summary.drift_drift_bundle_hosts,
            "drift_bundle_hosts_missing": summary.drift_drift_bundle_hosts_missing,
            "drift_status_valid_bundles": summary.drift_drift_status_valid_bundles,
            "drift_status_drifted_summaries": summary.drift_drift_status_drifted_summaries,
            "drift_status_missing_artifacts": summary.drift_drift_status_missing_artifacts,
            "drift_status_other_failures": summary.drift_drift_status_other_failures,
        },
    })
}

fn first_rollup_check_log_summary_verification_rollup_verification_summary_drift(
    expected: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
    actual: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummary,
) -> Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift>
{
    let fields = [
        ("records", expected.records, actual.records),
        ("matched", expected.matched, actual.matched),
        ("drifted", expected.drifted, actual.drifted),
        (
            "drift_records",
            expected.drift_records,
            actual.drift_records,
        ),
        (
            "drift_matched",
            expected.drift_matched,
            actual.drift_matched,
        ),
        (
            "drift_drifted",
            expected.drift_drifted,
            actual.drift_drifted,
        ),
        (
            "drift_drift_records",
            expected.drift_drift_records,
            actual.drift_drift_records,
        ),
        (
            "drift_drift_rollup_hosts_present",
            expected.drift_drift_rollup_hosts_present,
            actual.drift_drift_rollup_hosts_present,
        ),
        (
            "drift_drift_rollup_hosts_missing",
            expected.drift_drift_rollup_hosts_missing,
            actual.drift_drift_rollup_hosts_missing,
        ),
        (
            "drift_drift_bundle_hosts",
            expected.drift_drift_bundle_hosts,
            actual.drift_drift_bundle_hosts,
        ),
        (
            "drift_drift_bundle_hosts_missing",
            expected.drift_drift_bundle_hosts_missing,
            actual.drift_drift_bundle_hosts_missing,
        ),
        (
            "drift_drift_status_valid_bundles",
            expected.drift_drift_status_valid_bundles,
            actual.drift_drift_status_valid_bundles,
        ),
        (
            "drift_drift_status_drifted_summaries",
            expected.drift_drift_status_drifted_summaries,
            actual.drift_drift_status_drifted_summaries,
        ),
        (
            "drift_drift_status_missing_artifacts",
            expected.drift_drift_status_missing_artifacts,
            actual.drift_drift_status_missing_artifacts,
        ),
        (
            "drift_drift_status_other_failures",
            expected.drift_drift_status_other_failures,
            actual.drift_drift_status_other_failures,
        ),
    ];
    fields
        .into_iter()
        .find(|(_, expected, actual)| expected != actual)
        .map(|(field, expected, actual)| {
            rollup_check_log_summary_verification_rollup_verification_summary_drift(
                field, expected, actual,
            )
        })
}

fn rollup_check_log_summary_verification_rollup_verification_summary_drift(
    field: &'static str,
    expected: u64,
    actual: u64,
) -> RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift{
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerificationSummaryDrift {
        field,
        expected,
        actual,
    }
}

fn first_rollup_check_log_summary_verification_rollup_drift(
    expected: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
    actual: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
) -> Option<RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift>
{
    if expected.records != actual.records {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "records",
            expected.records,
            actual.records,
        ));
    }
    if expected.matched != actual.matched {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "matched",
            expected.matched,
            actual.matched,
        ));
    }
    if expected.drifted != actual.drifted {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drifted",
            expected.drifted,
            actual.drifted,
        ));
    }
    if expected.drift_records != actual.drift_records {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_records",
            expected.drift_records,
            actual.drift_records,
        ));
    }
    if expected.drift_rollup_hosts_present != actual.drift_rollup_hosts_present {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_rollup_hosts_present",
            expected.drift_rollup_hosts_present,
            actual.drift_rollup_hosts_present,
        ));
    }
    if expected.drift_rollup_hosts_missing != actual.drift_rollup_hosts_missing {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_rollup_hosts_missing",
            expected.drift_rollup_hosts_missing,
            actual.drift_rollup_hosts_missing,
        ));
    }
    if expected.drift_bundle_hosts != actual.drift_bundle_hosts {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_bundle_hosts",
            expected.drift_bundle_hosts,
            actual.drift_bundle_hosts,
        ));
    }
    if expected.drift_bundle_hosts_missing != actual.drift_bundle_hosts_missing {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_bundle_hosts_missing",
            expected.drift_bundle_hosts_missing,
            actual.drift_bundle_hosts_missing,
        ));
    }
    if expected.drift_status_valid_bundles != actual.drift_status_valid_bundles {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_status_valid_bundles",
            expected.drift_status_valid_bundles,
            actual.drift_status_valid_bundles,
        ));
    }
    if expected.drift_status_drifted_summaries != actual.drift_status_drifted_summaries {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_status_drifted_summaries",
            expected.drift_status_drifted_summaries,
            actual.drift_status_drifted_summaries,
        ));
    }
    if expected.drift_status_missing_artifacts != actual.drift_status_missing_artifacts {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_status_missing_artifacts",
            expected.drift_status_missing_artifacts,
            actual.drift_status_missing_artifacts,
        ));
    }
    if expected.drift_status_other_failures != actual.drift_status_other_failures {
        return Some(rollup_check_log_summary_verification_rollup_drift(
            "drift_status_other_failures",
            expected.drift_status_other_failures,
            actual.drift_status_other_failures,
        ));
    }
    None
}

fn rollup_check_log_summary_verification_rollup_drift(
    field: &'static str,
    expected: u64,
    actual: u64,
) -> RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift {
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift {
        field,
        expected,
        actual,
    }
}

fn add_rollup_check_to_log_summary(
    summary: &mut RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    summary.records = checked_add_rollup_check_log_summary_count(summary.records, 1, "records")?;
    if check.rollup_host_present {
        summary.rollup_hosts_present = checked_add_rollup_check_log_summary_count(
            summary.rollup_hosts_present,
            1,
            "rollup_hosts_present",
        )?;
    } else {
        summary.rollup_hosts_missing = checked_add_rollup_check_log_summary_count(
            summary.rollup_hosts_missing,
            1,
            "rollup_hosts_missing",
        )?;
    }
    summary.bundle_hosts = checked_add_rollup_check_log_summary_count(
        summary.bundle_hosts,
        check.bundle_hosts,
        "bundle_hosts",
    )?;
    summary.bundle_hosts_missing = checked_add_rollup_check_log_summary_count(
        summary.bundle_hosts_missing,
        check.bundle_hosts_missing,
        "bundle_hosts_missing",
    )?;
    summary.status_valid_bundles = checked_add_rollup_check_log_summary_count(
        summary.status_valid_bundles,
        check.valid_bundles,
        "status_valid_bundles",
    )?;
    summary.status_drifted_summaries = checked_add_rollup_check_log_summary_count(
        summary.status_drifted_summaries,
        check.drifted_summaries,
        "status_drifted_summaries",
    )?;
    summary.status_missing_artifacts = checked_add_rollup_check_log_summary_count(
        summary.status_missing_artifacts,
        check.missing_artifacts,
        "status_missing_artifacts",
    )?;
    summary.status_other_failures = checked_add_rollup_check_log_summary_count(
        summary.status_other_failures,
        check.other_failures,
        "status_other_failures",
    )?;
    Ok(())
}

fn require_rollup_check_json_schema(
    value: &Value,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let schema = rollup_required_str(value, "schema")?;
    if schema != REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(
                schema.to_owned(),
            ),
        );
    }
    Ok(())
}

fn parse_rollup_check_json_flat_fields(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
    RemoteFreeServiceTelemetryCollectionSummaryRollupError,
> {
    let path = rollup_required_str(value, "path")?.to_owned();
    Ok(RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
        path: PathBuf::from(&path),
        schema: rollup_required_str(value, "rollup_schema")?.to_owned(),
        artifact_bytes: rollup_required_u64(value, "artifact_bytes")?,
        artifact_fingerprint: rollup_required_str(value, "artifact_fingerprint")?.to_owned(),
        summaries: rollup_required_u64(value, "summaries")?,
        valid_bundles: rollup_required_u64(value, "valid_bundles")?,
        drifted_summaries: rollup_required_u64(value, "drifted_summaries")?,
        missing_artifacts: rollup_required_u64(value, "missing_artifacts")?,
        other_failures: rollup_required_u64(value, "other_failures")?,
        timing_ranges: rollup_required_u64(value, "timing_ranges")?,
        bundles: rollup_required_u64(value, "bundles")?,
        rollup_host_present: rollup_required_bool(value, "rollup_host_present")?,
        bundle_hosts: rollup_required_u64(value, "bundle_hosts")?,
        bundle_hosts_missing: rollup_required_u64(value, "bundle_hosts_missing")?,
    })
}

fn require_rollup_check_json_top_level_status(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    require_rollup_count(
        "status_valid_bundles",
        check.valid_bundles,
        rollup_required_u64(value, "status_valid_bundles")?,
    )?;
    require_rollup_count(
        "status_drifted_summaries",
        check.drifted_summaries,
        rollup_required_u64(value, "status_drifted_summaries")?,
    )?;
    require_rollup_count(
        "status_missing_artifacts",
        check.missing_artifacts,
        rollup_required_u64(value, "status_missing_artifacts")?,
    )?;
    require_rollup_count(
        "status_other_failures",
        check.other_failures,
        rollup_required_u64(value, "status_other_failures")?,
    )?;
    Ok(())
}

fn require_rollup_check_json_groups(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    require_rollup_check_json_artifact_group(value, check)?;
    require_rollup_check_json_counts_group(value, check)?;
    require_rollup_check_json_host_coverage_group(value, check)?;
    require_rollup_check_json_status_coverage_group(value, check)?;
    Ok(())
}

fn require_rollup_check_json_artifact_group(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let artifact = rollup_required_object(value, "artifact")?;
    require_rollup_json_field(
        "artifact.path",
        &check.path.display().to_string(),
        rollup_required_str(artifact, "path")?,
    )?;
    require_rollup_json_field(
        "artifact.rollup_schema",
        check.schema.as_str(),
        rollup_required_str(artifact, "rollup_schema")?,
    )?;
    require_rollup_count(
        "artifact.bytes",
        check.artifact_bytes,
        rollup_required_u64(artifact, "bytes")?,
    )?;
    require_rollup_json_field(
        "artifact.fingerprint",
        check.artifact_fingerprint.as_str(),
        rollup_required_str(artifact, "fingerprint")?,
    )?;
    Ok(())
}

fn require_rollup_check_json_counts_group(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let counts = rollup_required_object(value, "counts")?;
    require_rollup_count(
        "counts.summaries",
        check.summaries,
        rollup_required_u64(counts, "summaries")?,
    )?;
    require_rollup_count(
        "counts.valid_bundles",
        check.valid_bundles,
        rollup_required_u64(counts, "valid_bundles")?,
    )?;
    require_rollup_count(
        "counts.timing_ranges",
        check.timing_ranges,
        rollup_required_u64(counts, "timing_ranges")?,
    )?;
    require_rollup_count(
        "counts.bundles",
        check.bundles,
        rollup_required_u64(counts, "bundles")?,
    )?;
    Ok(())
}

fn require_rollup_check_json_host_coverage_group(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let host_coverage = rollup_required_object(value, "host_coverage")?;
    require_rollup_json_field(
        "host_coverage.rollup_host_present",
        &check.rollup_host_present.to_string(),
        &rollup_required_bool(host_coverage, "rollup_host_present")?.to_string(),
    )?;
    require_rollup_count(
        "host_coverage.bundle_hosts",
        check.bundle_hosts,
        rollup_required_u64(host_coverage, "bundle_hosts")?,
    )?;
    require_rollup_count(
        "host_coverage.bundle_hosts_missing",
        check.bundle_hosts_missing,
        rollup_required_u64(host_coverage, "bundle_hosts_missing")?,
    )?;
    Ok(())
}

fn require_rollup_check_json_status_coverage_group(
    value: &Value,
    check: &RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let status_coverage = rollup_required_object(value, "status_coverage")?;
    require_rollup_count(
        "status_coverage.valid_bundles",
        check.valid_bundles,
        rollup_required_u64(status_coverage, "valid_bundles")?,
    )?;
    require_rollup_count(
        "status_coverage.drifted_summaries",
        check.drifted_summaries,
        rollup_required_u64(status_coverage, "drifted_summaries")?,
    )?;
    require_rollup_count(
        "status_coverage.missing_artifacts",
        check.missing_artifacts,
        rollup_required_u64(status_coverage, "missing_artifacts")?,
    )?;
    require_rollup_count(
        "status_coverage.other_failures",
        check.other_failures,
        rollup_required_u64(status_coverage, "other_failures")?,
    )?;
    Ok(())
}

fn resolve_artifact_path_by_kind(
    summary_path: &Path,
    summary: &RemoteFreeServiceTelemetryCollectionSummary,
    kind: &str,
    missing_error: RemoteFreeServiceTelemetryCollectionSummaryError,
) -> Result<PathBuf, RemoteFreeServiceTelemetryCollectionSummaryError> {
    let base_dir = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let artifact = summary
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
        .ok_or(missing_error)?;

    resolve_summary_artifact_path(base_dir, &artifact.path)
}

fn parse_source(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummarySource,
    RemoteFreeServiceTelemetryCollectionSummaryError,
> {
    Ok(RemoteFreeServiceTelemetryCollectionSummarySource {
        role: required_str(value, "role")?.to_owned(),
        label: required_str(value, "label")?.to_owned(),
        input: required_str(value, "input")?.to_owned(),
        artifact: required_str(value, "artifact")?.to_owned(),
    })
}

fn summary_host_json(host: &RemoteFreeServiceTelemetryCollectionSummaryHost) -> Value {
    json!({
        "os": host.os.as_str(),
        "arch": host.arch.as_str(),
        "hostname": host.hostname.as_deref(),
    })
}

fn optional_summary_host(
    value: &Value,
) -> Result<
    Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
    RemoteFreeServiceTelemetryCollectionSummaryError,
> {
    let Some(host) = value.get("host") else {
        return Ok(None);
    };
    let host_value = host;
    if !host_value.is_object() {
        return Err(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType("host"));
    }
    Ok(Some(RemoteFreeServiceTelemetryCollectionSummaryHost {
        os: required_str(host_value, "os")?.to_owned(),
        arch: required_str(host_value, "arch")?.to_owned(),
        hostname: optional_nullable_str(host_value, "hostname")?.map(str::to_owned),
    }))
}

fn parse_artifact(
    value: &Value,
) -> Result<
    RemoteFreeServiceTelemetryCollectionSummaryArtifact,
    RemoteFreeServiceTelemetryCollectionSummaryError,
> {
    Ok(RemoteFreeServiceTelemetryCollectionSummaryArtifact {
        kind: required_str(value, "kind")?.to_owned(),
        role: optional_str(value, "role")?.map(str::to_owned),
        path: required_str(value, "path")?.to_owned(),
        byte_count: required_u64(value, "byte_count")?,
    })
}

fn required_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, RemoteFreeServiceTelemetryCollectionSummaryError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::MissingField(field))?
        .as_str()
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn optional_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<Option<&'a str>, RemoteFreeServiceTelemetryCollectionSummaryError> {
    let Some(value) = value.get(field) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(Some)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn optional_nullable_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<Option<&'a str>, RemoteFreeServiceTelemetryCollectionSummaryError> {
    let Some(value) = value.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(Some)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn required_array<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a [Value], RemoteFreeServiceTelemetryCollectionSummaryError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::MissingField(field))?
        .as_array()
        .map(Vec::as_slice)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn required_string_array(
    value: &Value,
    field: &'static str,
) -> Result<Vec<String>, RemoteFreeServiceTelemetryCollectionSummaryError> {
    required_array(value, field)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
        })
        .collect()
}

fn required_u64(
    value: &Value,
    field: &'static str,
) -> Result<u64, RemoteFreeServiceTelemetryCollectionSummaryError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::MissingField(field))?
        .as_u64()
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn required_usize(
    value: &Value,
    field: &'static str,
) -> Result<usize, RemoteFreeServiceTelemetryCollectionSummaryError> {
    let value = required_u64(value, field)?;
    value
        .try_into()
        .map_err(|_| RemoteFreeServiceTelemetryCollectionSummaryError::InvalidFieldType(field))
}

fn rollup_required_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(field))?
        .as_str()
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType(field))
}

fn rollup_required_array<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a [Value], RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(field))?
        .as_array()
        .map(Vec::as_slice)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType(field))
}

fn rollup_required_object<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a Value, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    let object = value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(field))?;
    if object.is_object() {
        Ok(object)
    } else {
        Err(RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType(field))
    }
}

fn rollup_required_u64(
    value: &Value,
    field: &'static str,
) -> Result<u64, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(field))?
        .as_u64()
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType(field))
}

fn rollup_required_bool(
    value: &Value,
    field: &'static str,
) -> Result<bool, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    value
        .get(field)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(field))?
        .as_bool()
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::InvalidFieldType(field))
}

fn require_rollup_count(
    field: &'static str,
    expected: u64,
    actual: u64,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if expected == actual {
        return Ok(());
    }

    Err(
        RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
            field,
            expected,
            actual,
        },
    )
}

fn require_rollup_json_field(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    if expected == actual {
        Ok(())
    } else {
        Err(
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field,
                expected: expected.to_owned(),
                actual: actual.to_owned(),
            },
        )
    }
}

fn checked_add_rollup_check_log_summary_count(
    lhs: u64,
    rhs: u64,
    field: &'static str,
) -> Result<u64, RemoteFreeServiceTelemetryCollectionSummaryRollupError> {
    lhs.checked_add(rhs)
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountOverflow(field))
}

fn rollup_artifact_fingerprint(text: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("fnv1a64:{hash:016x}")
}

fn usize_to_u64(
    value: usize,
    field: &'static str,
) -> Result<u64, RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError> {
    value.try_into().map_err(|_| {
        RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError::CountOverflow(field)
    })
}

fn checked_add_rollup_count(
    lhs: u64,
    rhs: u64,
    field: &'static str,
) -> Result<u64, RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError> {
    lhs.checked_add(rhs).ok_or(
        RemoteFreeServiceTelemetryCollectionSummaryDirectoryRollupError::CountOverflow(field),
    )
}

fn collect_summary_paths_recursive(
    root: &Path,
    summary_paths: &mut Vec<PathBuf>,
) -> Result<(), RemoteFreeServiceTelemetryCollectionSummaryScanError> {
    let entries = fs::read_dir(root).map_err(|source| {
        RemoteFreeServiceTelemetryCollectionSummaryScanError {
            path: root.to_path_buf(),
            source,
        }
    })?;
    for entry in entries {
        let entry =
            entry.map_err(
                |source| RemoteFreeServiceTelemetryCollectionSummaryScanError {
                    path: root.to_path_buf(),
                    source,
                },
            )?;
        let path = entry.path();
        if path.is_dir() {
            collect_summary_paths_recursive(&path, summary_paths)?;
        } else if path
            .file_name()
            .is_some_and(|file_name| file_name == "collection-summary.json")
        {
            summary_paths.push(path);
        }
    }
    Ok(())
}

fn resolve_summary_artifact_path(
    base_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, RemoteFreeServiceTelemetryCollectionSummaryError> {
    let path = Path::new(relative_path);
    if path.is_absolute() || relative_path.is_empty() {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryError::InvalidArtifactPath(
                relative_path.to_owned(),
            ),
        );
    }

    if !path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(
            RemoteFreeServiceTelemetryCollectionSummaryError::InvalidArtifactPath(
                relative_path.to_owned(),
            ),
        );
    }

    Ok(base_dir.join(path))
}

#[cfg(test)]
mod tests {
    use super::{
        build_remote_free_service_telemetry_collection_summary_directory_rollup,
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log,
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log,
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log,
        collect_remote_free_service_telemetry_collection_summary_paths,
        format_remote_free_service_telemetry_collection_summary_rollup_check_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line,
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line,
        parse_remote_free_service_telemetry_collection_summary,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line,
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_log,
        resolve_remote_free_service_telemetry_collection_summary_manifest_path,
        resolve_remote_free_service_telemetry_collection_summary_validation_summary_path,
        rollup_artifact_fingerprint,
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log,
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log,
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log,
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log,
        validate_remote_free_service_telemetry_collection_summary_rollup_artifact,
        verify_remote_free_service_telemetry_collection_summary_artifacts,
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log,
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log,
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log,
        write_remote_free_service_telemetry_collection_summary_rollup_artifact,
        RemoteFreeServiceTelemetryCollectionSummaryBundleValidation,
        RemoteFreeServiceTelemetryCollectionSummaryError,
        RemoteFreeServiceTelemetryCollectionSummaryHost,
        RemoteFreeServiceTelemetryCollectionSummaryRollup,
        RemoteFreeServiceTelemetryCollectionSummaryRollupBundle,
        RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheck,
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup,
        RemoteFreeServiceTelemetryCollectionSummaryRollupError,
        RemoteFreeServiceTelemetryCollectionSummaryRollupHost,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA,
    };
    use serde_json::json;
    use std::{
        env, fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn summary_json(output_byte_count: u64, manifest_path: &str) -> String {
        format!(
            r#"{{
  "schema": "locus.remote_free_service.telemetry.collection_summary.v1",
  "collection_mode": "benchmark_capture",
  "run_id": "run-1",
  "output_count": 1,
  "criterion_args": ["--sample-size", "10"],
  "sources": [
    {{
      "role": "baseline",
      "label": "run-01",
      "input": "benchmark",
      "artifact": "run-01.txt"
    }}
  ],
  "artifacts": [
    {{
      "kind": "output",
      "role": "baseline",
      "path": "run-01.txt",
      "byte_count": {output_byte_count}
    }},
    {{
      "kind": "manifest",
      "path": "{manifest_path}",
      "byte_count": 8
    }}
  ]
}}"#
        )
    }

    fn summary_json_with_host() -> String {
        let mut summary =
            serde_json::from_str::<serde_json::Value>(&summary_json(3, "manifest.txt"))
                .expect("summary json");
        summary.as_object_mut().expect("summary object").insert(
            "host".to_owned(),
            json!({
                "os": "linux",
                "arch": "x86_64",
                "hostname": "bench-host-01"
            }),
        );
        format!(
            "{}\n",
            serde_json::to_string_pretty(&summary).expect("json")
        )
    }

    fn summary_json_with_null_hostname() -> String {
        let mut summary =
            serde_json::from_str::<serde_json::Value>(&summary_json_with_host()).expect("summary");
        summary["host"]["hostname"] = serde_json::Value::Null;
        format!(
            "{}\n",
            serde_json::to_string_pretty(&summary).expect("json")
        )
    }

    fn rollup_json(status: &str, valid_bundles: u64, drifted_summaries: u64) -> String {
        let artifact = json!({
            "schema": "locus.remote_free_service.telemetry.collection_summary_rollup.v2",
            "root": "evidence",
            "summaries": 1,
            "valid_bundles": valid_bundles,
            "drifted_summaries": drifted_summaries,
            "missing_artifacts": 0,
            "other_failures": 0,
            "timing_ranges": 1,
            "bundles": [
                {
                    "summary": "run-1/collection-summary.json",
                    "run_id": "run-1",
                    "status": status,
                    "timing_ranges": 1
                }
            ]
        });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact).expect("json")
        )
    }

    fn rollup_json_with_host(status: &str, valid_bundles: u64, drifted_summaries: u64) -> String {
        let mut artifact = serde_json::from_str::<serde_json::Value>(&rollup_json(
            status,
            valid_bundles,
            drifted_summaries,
        ))
        .expect("rollup json");
        artifact.as_object_mut().expect("rollup object").insert(
            "host".to_owned(),
            json!({
                "os": "linux",
                "arch": "x86_64",
                "hostname": "bench-host-01"
            }),
        );
        artifact["bundles"][0]["host"] = json!({
            "os": "linux",
            "arch": "x86_64",
            "hostname": "bench-host-01"
        });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact).expect("json")
        )
    }

    fn rollup_json_with_invalid_host_metadata() -> String {
        let mut artifact = serde_json::from_str::<serde_json::Value>(&rollup_json("valid", 1, 0))
            .expect("rollup json");
        artifact["host"] = json!("not-host-object");
        artifact["bundles"][0]["host"] = json!("not-host-object");
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact).expect("json")
        )
    }

    fn rollup_json_with_unexpected_schema() -> String {
        let mut artifact = serde_json::from_str::<serde_json::Value>(&rollup_json("valid", 1, 0))
            .expect("rollup json");
        artifact["schema"] =
            json!("locus.remote_free_service.telemetry.collection_summary_rollup.v1");
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact).expect("json")
        )
    }

    fn rollup_json_with_valid_and_drifted_host_rows() -> String {
        let artifact = json!({
            "schema": "locus.remote_free_service.telemetry.collection_summary_rollup.v2",
            "root": "evidence",
            "host": {
                "os": "linux",
                "arch": "x86_64",
                "hostname": "bench-host-01"
            },
            "summaries": 2,
            "valid_bundles": 1,
            "drifted_summaries": 1,
            "missing_artifacts": 0,
            "other_failures": 0,
            "timing_ranges": 1,
            "bundles": [
                {
                    "summary": "run-1/collection-summary.json",
                    "run_id": "run-1",
                    "host": {
                        "os": "linux",
                        "arch": "x86_64",
                        "hostname": "bench-host-01"
                    },
                    "status": "valid",
                    "timing_ranges": 1
                },
                {
                    "summary": "run-2/collection-summary.json",
                    "run_id": "run-2",
                    "host": {
                        "os": "linux",
                        "arch": "x86_64",
                        "hostname": "bench-host-01"
                    },
                    "status": "drifted_summary",
                    "timing_ranges": 0
                }
            ]
        });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact).expect("json")
        )
    }

    #[test]
    fn rollup_artifact_fingerprint_is_stable_and_content_sensitive() {
        let artifact_text = rollup_json("valid", 1, 0);

        assert_eq!(
            rollup_artifact_fingerprint(&artifact_text),
            rollup_artifact_fingerprint(&artifact_text)
        );
        assert_ne!(
            rollup_artifact_fingerprint(&artifact_text),
            rollup_artifact_fingerprint(&rollup_json_with_host("valid", 1, 0))
        );
        assert!(rollup_artifact_fingerprint(&artifact_text).starts_with("fnv1a64:"));
        assert_eq!(rollup_artifact_fingerprint(&artifact_text).len(), 24);
    }

    fn temp_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let dir = env::temp_dir().join(format!(
            "locus-summary-validate-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir)?;
        Ok(dir)
    }

    fn sample_rollup_check() -> RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
            path: std::path::PathBuf::from("target/locus-evidence/sample-rollup.json"),
            schema: "locus.remote_free_service.telemetry.collection_summary_rollup.v2".to_owned(),
            artifact_bytes: 694,
            artifact_fingerprint: "fnv1a64:82185294cde2c506".to_owned(),
            summaries: 1,
            valid_bundles: 1,
            drifted_summaries: 0,
            missing_artifacts: 0,
            other_failures: 0,
            timing_ranges: 1,
            bundles: 1,
            rollup_host_present: true,
            bundle_hosts: 1,
            bundle_hosts_missing: 0,
        }
    }

    fn sample_rollup_check_without_bundle_host(
    ) -> RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
        RemoteFreeServiceTelemetryCollectionSummaryRollupCheck {
            path: std::path::PathBuf::from("target/locus-evidence/no-host-rollup.json"),
            schema: "locus.remote_free_service.telemetry.collection_summary_rollup.v2".to_owned(),
            artifact_bytes: 591,
            artifact_fingerprint: "fnv1a64:f788b8ab364b6e1b".to_owned(),
            summaries: 1,
            valid_bundles: 1,
            drifted_summaries: 0,
            missing_artifacts: 0,
            other_failures: 0,
            timing_ranges: 1,
            bundles: 1,
            rollup_host_present: true,
            bundle_hosts: 0,
            bundle_hosts_missing: 1,
        }
    }

    fn sample_rollup_check_json_source_log() -> Result<String, Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let second = sample_rollup_check_without_bundle_host();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let second_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                &second,
            )?;
        Ok(format!("{first_line}\n{second_line}\n"))
    }

    fn sample_rollup_check_log_summary_json(
        source_log: &str,
    ) -> Result<
        (
            RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
            String,
        ),
        Box<dyn std::error::Error>,
    > {
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        Ok((summary, json_line))
    }

    fn rollup_validation_for_bundle(
        bundle: &str,
    ) -> RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
        match bundle {
            "valid" => RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                run_id: Some("valid".to_owned()),
                host: Some(RemoteFreeServiceTelemetryCollectionSummaryHost {
                    os: "linux".to_owned(),
                    arch: "x86_64".to_owned(),
                    hostname: Some("bench-host-01".to_owned()),
                }),
                status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid,
                timing_ranges: 2,
            },
            "drifted" => RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                run_id: Some("drifted".to_owned()),
                host: None,
                status:
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::DriftedSummary,
                timing_ranges: 0,
            },
            "missing" => RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                run_id: Some("missing".to_owned()),
                host: None,
                status:
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::MissingArtifact,
                timing_ranges: 0,
            },
            _ => RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                run_id: None,
                host: None,
                status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::OtherFailure,
                timing_ranges: 0,
            },
        }
    }

    #[test]
    fn parses_collection_summary() {
        let summary = parse_remote_free_service_telemetry_collection_summary(&summary_json(
            3,
            "manifest.txt",
        ))
        .expect("summary");

        assert_eq!(summary.collection_mode, "benchmark_capture");
        assert_eq!(summary.run_id, "run-1");
        assert_eq!(summary.host, None);
        assert_eq!(summary.output_count, 1);
        assert_eq!(summary.criterion_args, ["--sample-size", "10"]);
        assert_eq!(summary.sources[0].artifact, "run-01.txt");
        assert_eq!(summary.artifacts[0].byte_count, 3);
    }

    #[test]
    fn parses_collection_summary_host_metadata() {
        let summary =
            parse_remote_free_service_telemetry_collection_summary(&summary_json_with_host())
                .expect("summary");

        let host = summary.host.expect("host metadata");
        assert_eq!(host.os, "linux");
        assert_eq!(host.arch, "x86_64");
        assert_eq!(host.hostname.as_deref(), Some("bench-host-01"));
    }

    #[test]
    fn parses_collection_summary_host_metadata_without_hostname() {
        let summary = parse_remote_free_service_telemetry_collection_summary(
            &summary_json_with_null_hostname(),
        )
        .expect("summary");

        let host = summary.host.expect("host metadata");
        assert_eq!(host.os, "linux");
        assert_eq!(host.arch, "x86_64");
        assert_eq!(host.hostname, None);
    }

    #[test]
    fn rejects_output_count_mismatch() {
        let input =
            summary_json(3, "manifest.txt").replace("\"output_count\": 1", "\"output_count\": 2");
        let error = parse_remote_free_service_telemetry_collection_summary(&input)
            .expect_err("output mismatch");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryError::OutputCountMismatch { .. }
        ));
    }

    #[test]
    fn scans_collection_summary_paths_sorted() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let a = dir.join("a");
        let z = dir.join("z").join("nested");
        fs::create_dir_all(&a)?;
        fs::create_dir_all(&z)?;
        fs::write(a.join("collection-summary.json"), "{}")?;
        fs::write(z.join("collection-summary.json"), "{}")?;
        fs::write(dir.join("collection-summary-rollup.json"), "{}")?;
        fs::write(dir.join("not-a-summary.json"), "{}")?;

        let paths = collect_remote_free_service_telemetry_collection_summary_paths(&dir)?;
        let relative_paths = paths
            .iter()
            .map(|path| {
                path.strip_prefix(&dir)
                    .expect("under root")
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            relative_paths,
            vec![
                "a/collection-summary.json",
                "z/nested/collection-summary.json"
            ]
        );
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn builds_collection_summary_directory_rollup_from_validator(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        fs::create_dir_all(dir.join("valid"))?;
        fs::create_dir_all(dir.join("drifted"))?;
        fs::create_dir_all(dir.join("missing"))?;
        fs::write(dir.join("valid").join("collection-summary.json"), "{}")?;
        fs::write(dir.join("drifted").join("collection-summary.json"), "{}")?;
        fs::write(dir.join("missing").join("collection-summary.json"), "{}")?;
        fs::write(dir.join("collection-summary-rollup.json"), "{}")?;

        let rollup = build_remote_free_service_telemetry_collection_summary_directory_rollup(
            &dir,
            |summary_path| {
                let bundle = summary_path
                    .parent()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .expect("bundle name");
                rollup_validation_for_bundle(bundle)
            },
        )?;

        assert_eq!(rollup.root, dir);
        assert_eq!(rollup.summaries, 3);
        assert_eq!(rollup.valid_bundles, 1);
        assert_eq!(rollup.drifted_summaries, 1);
        assert_eq!(rollup.missing_artifacts, 1);
        assert_eq!(rollup.other_failures, 0);
        assert_eq!(rollup.timing_ranges, 2);
        assert_eq!(
            rollup.to_string(),
            format!(
                "remote_free_service_telemetry_collection_summary_rollup root={} summaries=3 valid_bundles=1 drifted_summaries=1 missing_artifacts=1 other_failures=0 timing_ranges=2",
                rollup.root.display()
            )
        );
        let bundle_rows = rollup
            .bundles
            .iter()
            .map(|bundle| {
                (
                    bundle.summary.as_str(),
                    bundle.run_id.as_deref(),
                    bundle.status.as_str(),
                    bundle.timing_ranges,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bundle_rows,
            vec![
                (
                    "drifted/collection-summary.json",
                    Some("drifted"),
                    "drifted_summary",
                    0
                ),
                (
                    "missing/collection-summary.json",
                    Some("missing"),
                    "missing_artifact",
                    0
                ),
                ("valid/collection-summary.json", Some("valid"), "valid", 2,),
            ]
        );
        let valid_bundle = rollup
            .bundles
            .iter()
            .find(|bundle| bundle.summary == "valid/collection-summary.json")
            .expect("valid bundle");
        let host = valid_bundle.host.as_ref().expect("host metadata");
        assert_eq!(host.os, "linux");
        assert_eq!(host.arch, "x86_64");
        assert_eq!(host.hostname.as_deref(), Some("bench-host-01"));
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn verifies_artifact_byte_counts() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        fs::write(dir.join("run-01.txt"), "abc")?;
        fs::write(dir.join("manifest.txt"), "manifest")?;
        let summary_path = dir.join("collection-summary.json");
        fs::write(&summary_path, summary_json(3, "manifest.txt"))?;
        let summary = parse_remote_free_service_telemetry_collection_summary(&fs::read_to_string(
            &summary_path,
        )?)?;

        let report = verify_remote_free_service_telemetry_collection_summary_artifacts(
            &summary_path,
            &summary,
        )?;

        assert_eq!(report.verified_artifacts, 2);
        assert_eq!(report.verified_bytes, 11);
        assert_eq!(
            report.to_string(),
            "remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=2 verified_bytes=11"
        );
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn rejects_artifact_byte_count_mismatch() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        fs::write(dir.join("run-01.txt"), "abcd")?;
        fs::write(dir.join("manifest.txt"), "manifest")?;
        let summary_path = dir.join("collection-summary.json");
        fs::write(&summary_path, summary_json(3, "manifest.txt"))?;
        let summary = parse_remote_free_service_telemetry_collection_summary(&fs::read_to_string(
            &summary_path,
        )?)?;

        let error = verify_remote_free_service_telemetry_collection_summary_artifacts(
            &summary_path,
            &summary,
        )
        .expect_err("byte mismatch");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryError::ByteCountMismatch { .. }
        ));
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn rejects_artifact_path_traversal() {
        let summary = parse_remote_free_service_telemetry_collection_summary(&summary_json(
            3,
            "../manifest.txt",
        ))
        .expect("summary");
        let error = resolve_remote_free_service_telemetry_collection_summary_manifest_path(
            std::path::Path::new("/tmp/collection-summary.json"),
            &summary,
        )
        .expect_err("path traversal");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryError::InvalidArtifactPath(_)
        ));
    }

    #[test]
    fn reports_missing_validation_summary_artifact() {
        let summary = parse_remote_free_service_telemetry_collection_summary(&summary_json(
            3,
            "manifest.txt",
        ))
        .expect("summary");
        let error =
            resolve_remote_free_service_telemetry_collection_summary_validation_summary_path(
                std::path::Path::new("/tmp/collection-summary.json"),
                &summary,
            )
            .expect_err("missing validation summary");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryError::MissingValidationSummaryArtifact
        ));
    }

    #[test]
    fn validates_collection_summary_rollup_artifact() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        let artifact_text = rollup_json("valid", 1, 0);
        fs::write(&rollup_path, &artifact_text)?;

        let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
            &rollup_path,
        )?;

        assert_eq!(check.path, rollup_path);
        assert_eq!(
            check.schema,
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(
            check.artifact_bytes,
            u64::try_from(artifact_text.len()).expect("artifact bytes")
        );
        assert_eq!(
            check.artifact_fingerprint,
            rollup_artifact_fingerprint(&artifact_text)
        );
        assert_eq!(check.summaries, 1);
        assert_eq!(check.valid_bundles, 1);
        assert_eq!(check.drifted_summaries, 0);
        assert_eq!(check.missing_artifacts, 0);
        assert_eq!(check.other_failures, 0);
        assert_eq!(check.timing_ranges, 1);
        assert_eq!(check.bundles, 1);
        assert!(!check.rollup_host_present);
        assert_eq!(check.bundle_hosts, 0);
        assert_eq!(check.bundle_hosts_missing, 1);
        assert_eq!(
            check.to_string(),
            format!(
                "remote_free_service_telemetry_collection_summary_rollup_check=ok path={} schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes={} artifact_fingerprint={} summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=false bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0",
                check.path.display(),
                artifact_text.len(),
                rollup_artifact_fingerprint(&artifact_text)
            )
        );
        let json_line_text =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        assert!(!json_line_text.contains('\n'));
        assert_eq!(
            parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                &json_line_text
            )?,
            check
        );
        let json_line = serde_json::from_str::<serde_json::Value>(&json_line_text)?;
        assert_eq!(
            json_line["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA
        );
        assert_eq!(
            json_line["rollup_schema"],
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(json_line["path"], check.path.display().to_string());
        assert_eq!(json_line["artifact_bytes"], check.artifact_bytes);
        assert_eq!(
            json_line["artifact_fingerprint"],
            check.artifact_fingerprint
        );
        assert_eq!(json_line["status_valid_bundles"], 1);
        assert_eq!(json_line["status_drifted_summaries"], 0);
        assert_eq!(
            json_line["artifact"]["path"],
            check.path.display().to_string()
        );
        assert_eq!(json_line["artifact"]["bytes"], check.artifact_bytes);
        assert_eq!(
            json_line["artifact"]["fingerprint"],
            check.artifact_fingerprint
        );
        assert_eq!(json_line["counts"]["summaries"], check.summaries);
        assert_eq!(json_line["counts"]["timing_ranges"], check.timing_ranges);
        assert_eq!(
            json_line["host_coverage"]["rollup_host_present"],
            check.rollup_host_present
        );
        assert_eq!(
            json_line["host_coverage"]["bundle_hosts_missing"],
            check.bundle_hosts_missing
        );
        assert_eq!(
            json_line["status_coverage"]["valid_bundles"],
            check.valid_bundles
        );
        assert_eq!(
            json_line["status_coverage"]["drifted_summaries"],
            check.drifted_summaries
        );
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn parses_rollup_check_json_line() -> Result<(), Box<dyn std::error::Error>> {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;

        let parsed = parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
            &json_line,
        )?;

        assert_eq!(parsed, check);
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_json_line_schema_drift() -> Result<(), Box<dyn std::error::Error>> {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["schema"] =
            json!("locus.remote_free_service.telemetry.collection_summary_rollup_check.v0");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&value)
                .expect_err("schema drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(_)
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_json_line_missing_group() -> Result<(), Box<dyn std::error::Error>> {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value
            .as_object_mut()
            .expect("json object")
            .remove("artifact");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&value)
                .expect_err("missing group");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("artifact")
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_json_line_grouped_count_drift() -> Result<(), Box<dyn std::error::Error>>
    {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["counts"]["valid_bundles"] = json!(2);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&value)
                .expect_err("grouped count drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "counts.valid_bundles",
                expected: 1,
                actual: 2
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_json_line_grouped_field_drift() -> Result<(), Box<dyn std::error::Error>>
    {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["artifact"]["fingerprint"] = json!("fnv1a64:0000000000000000");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&value)
                .expect_err("grouped field drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "artifact.fingerprint",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn summarizes_rollup_check_json_log() -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let second = sample_rollup_check_without_bundle_host();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let second_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                &second,
            )?;
        let log = format!("{first}\n{first_line}\n{second}\n{second_line}\n");

        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(&log)?;

        assert_eq!(summary.records, 2);
        assert_eq!(summary.rollup_hosts_present, 2);
        assert_eq!(summary.rollup_hosts_missing, 0);
        assert_eq!(summary.bundle_hosts, 1);
        assert_eq!(summary.bundle_hosts_missing, 1);
        assert_eq!(summary.status_valid_bundles, 2);
        assert_eq!(summary.status_drifted_summaries, 0);
        assert_eq!(summary.status_missing_artifacts, 0);
        assert_eq!(summary.status_other_failures, 0);
        assert_eq!(
            summary.to_string(),
            "remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0"
        );
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        assert!(!json_line.contains('\n'));
        assert_eq!(
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &json_line
            )?,
            summary
        );
        let json_line = serde_json::from_str::<serde_json::Value>(&json_line)?;
        assert_eq!(
            json_line["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SCHEMA
        );
        assert_eq!(json_line["records"], summary.records);
        assert_eq!(
            json_line["rollup_hosts_present"],
            summary.rollup_hosts_present
        );
        assert_eq!(json_line["bundle_hosts"], summary.bundle_hosts);
        assert_eq!(
            json_line["status_valid_bundles"],
            summary.status_valid_bundles
        );
        assert_eq!(
            json_line["host_coverage"]["bundle_hosts_missing"],
            summary.bundle_hosts_missing
        );
        assert_eq!(
            json_line["status_coverage"]["valid_bundles"],
            summary.status_valid_bundles
        );
        assert_eq!(
            json_line["status_coverage"]["other_failures"],
            summary.status_other_failures
        );
        Ok(())
    }

    #[test]
    fn parses_rollup_check_log_summary_json_line() -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let second = sample_rollup_check_without_bundle_host();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let second_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                &second,
            )?;
        let log = format!("{first_line}\n{second_line}\n");
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(&log)?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &json_line,
            )?;

        assert_eq!(parsed, summary);
        Ok(())
    }

    #[test]
    fn parses_rollup_check_log_summary_json_log() -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                &first_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        let log = format!(
            "remote_free_service_telemetry_collection_summary_rollup_check_log=ok\n{json_line}\n"
        );

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &log,
            )?;

        assert_eq!(parsed, summary);
        Ok(())
    }

    #[test]
    fn verifies_rollup_check_log_summary_json_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (summary, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let summary_log = format!("{summary}\n{json_line}\n");

        let verified =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;

        assert_eq!(verified, summary);
        Ok(())
    }

    #[test]
    fn formats_matched_rollup_check_log_summary_json_verification(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (summary, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let summary_log = format!("{summary}\n{json_line}\n");

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &report,
            )?;

        assert!(report.is_matched());
        assert_eq!(report.status_str(), "matched");
        assert!(!verdict_line.contains('\n'));
        let verdict = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        assert_eq!(
            verdict["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_SCHEMA
        );
        assert_eq!(verdict["status"], "matched");
        assert_eq!(verdict["matched"], true);
        assert!(verdict["drift"].is_null());
        assert_eq!(verdict["expected"]["records"], summary.records);
        assert_eq!(verdict["actual"]["records"], summary.records);
        assert_eq!(
            verdict["actual"]["host_coverage"]["bundle_hosts_missing"],
            summary.bundle_hosts_missing
        );
        assert_eq!(
            verdict["actual"]["status_coverage"]["valid_bundles"],
            summary.status_valid_bundles
        );
        Ok(())
    }

    #[test]
    fn parses_matched_rollup_check_log_summary_json_verification(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (summary, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let summary_log = format!("{summary}\n{json_line}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &report,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &verdict_line,
            )?;

        assert_eq!(parsed, report);
        assert!(parsed.is_matched());
        Ok(())
    }

    #[test]
    fn formats_drifted_rollup_check_log_summary_json_verification(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (_, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &report,
            )?;

        assert!(!report.is_matched());
        assert_eq!(report.status_str(), "drifted");
        assert_eq!(report.expected.records, 2);
        assert_eq!(report.actual.records, 1);
        assert_eq!(
            report.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        let verdict = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        assert_eq!(verdict["status"], "drifted");
        assert_eq!(verdict["matched"], false);
        assert_eq!(verdict["expected"]["records"], 2);
        assert_eq!(verdict["actual"]["records"], 1);
        assert_eq!(verdict["drift"]["field"], "records");
        assert_eq!(verdict["drift"]["expected"], 2);
        assert_eq!(verdict["drift"]["actual"], 1);
        Ok(())
    }

    #[test]
    fn parses_drifted_rollup_check_log_summary_json_verification(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (_, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &report,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &verdict_line,
            )?;

        assert_eq!(parsed, report);
        assert_eq!(
            parsed.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        Ok(())
    }

    #[test]
    fn summarizes_rollup_check_log_summary_json_verification_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log = sample_rollup_check_log_summary_json_verification_log()?;

        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;

        assert_eq!(rollup.records, 2);
        assert_eq!(rollup.matched, 1);
        assert_eq!(rollup.drifted, 1);
        assert_eq!(rollup.drift_records, 1);
        assert_eq!(rollup.drift_bundle_hosts, 0);
        assert!(!json_line.contains('\n'));
        let value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        assert_eq!(
            value["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_SCHEMA
        );
        assert_eq!(value["records"], 2);
        assert_eq!(value["status_coverage"]["matched"], 1);
        assert_eq!(value["status_coverage"]["drifted"], 1);
        assert_eq!(value["drift_fields"]["records"], 1);
        Ok(())
    }

    fn sample_rollup_check_log_summary_json_verification_log(
    ) -> Result<String, Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (summary, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let summary_log = format!("{summary}\n{json_line}\n");
        let matched_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let matched_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &matched_report,
            )?;
        let mut drifted_summary = serde_json::from_str::<serde_json::Value>(&json_line)?;
        drifted_summary["records"] = json!(1);
        let drifted_summary_log = serde_json::to_string(&drifted_summary)?;
        let drifted_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &drifted_summary_log,
            )?;
        let drifted_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &drifted_report,
            )?;
        Ok(format!("{matched_line}\n{drifted_line}\n"))
    }

    #[test]
    fn parses_rollup_check_log_summary_json_verification_rollup(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (summary, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let summary_log = format!("{summary}\n{json_line}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
                &report,
            )?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &verdict_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &json_line,
            )?;

        assert_eq!(parsed, rollup);
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_schema_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rollup =
            RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
                records: 1,
                matched: 1,
                ..Default::default()
            };
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["schema"] =
            json!("locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup.v0");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("schema drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(_)
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_missing_group(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rollup =
            RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
                records: 1,
                matched: 1,
                ..Default::default()
            };
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value
            .as_object_mut()
            .expect("json object")
            .remove("drift_fields");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("missing group");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("drift_fields")
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_status_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rollup =
            RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
                records: 1,
                matched: 1,
                ..Default::default()
            };
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["status_coverage"]["matched"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_coverage.matched",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_drift_field_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rollup =
            RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup {
                records: 1,
                drifted: 1,
                drift_records: 1,
                ..Default::default()
            };
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("drift field drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn verifies_rollup_check_log_summary_json_verification_rollup_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("dashboard rollup\n{json_line}\n");

        let verified =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &rollup_log,
            )?;

        assert_eq!(verified, rollup);
        assert_eq!(parsed, rollup);
        Ok(())
    }

    #[test]
    fn formats_matched_rollup_check_log_summary_json_verification_rollup_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("dashboard rollup\n{json_line}\n");

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;

        assert!(report.is_matched());
        assert_eq!(report.status_str(), "matched");
        assert_eq!(
            value["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA
        );
        assert_eq!(value["status"], "matched");
        assert_eq!(value["matched"], true);
        assert!(value["drift"].is_null());
        assert_eq!(value["expected"]["records"], 2);
        assert_eq!(value["actual"]["records"], 2);
        assert_eq!(value["expected"]["drift_fields"]["records"], 1);
        assert_eq!(value["actual"]["drift_fields"]["records"], 1);
        Ok(())
    }

    #[test]
    fn formats_drifted_rollup_check_log_summary_json_verification_rollup_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let rollup_log = serde_json::to_string(&value)?;

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        let strict_error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("strict verifier rejects records drift");

        assert!(!report.is_matched());
        assert_eq!(report.status_str(), "drifted");
        assert_eq!(
            report.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        assert_eq!(value["status"], "drifted");
        assert_eq!(value["matched"], false);
        assert_eq!(value["expected"]["records"], 2);
        assert_eq!(value["actual"]["records"], 1);
        assert_eq!(value["drift"]["field"], "records");
        assert_eq!(value["drift"]["expected"], 2);
        assert_eq!(value["drift"]["actual"], 1);
        assert!(matches!(
            strict_error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn parses_matched_rollup_check_log_summary_json_verification_rollup_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("dashboard rollup\n{json_line}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let verdict_log = format!("published verdict\n{verdict_line}\n");

        let parsed_line =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &verdict_line,
            )?;
        let parsed_log =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
                &verdict_log,
            )?;

        assert_eq!(parsed_line, report);
        assert_eq!(parsed_log, report);
        assert!(parsed_log.is_matched());
        Ok(())
    }

    #[test]
    fn parses_drifted_rollup_check_log_summary_json_verification_rollup_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let rollup_log = serde_json::to_string(&value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &verdict_line,
            )?;

        assert_eq!(parsed, report);
        assert!(!parsed.is_matched());
        assert_eq!(
            parsed.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_status_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("{json_line}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["status"] = json!("drifted");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "status",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_drift_payload_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let rollup_log = serde_json::to_string(&value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["drift"]["actual"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("drift payload drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_nested_group_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("{json_line}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["expected"]["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("nested group drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn summarizes_rollup_check_log_summary_json_verification_rollup_verdicts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &summary_json,
            )?;

        assert_eq!(parsed.records, 2);
        assert_eq!(parsed.matched, 1);
        assert_eq!(parsed.drifted, 1);
        assert_eq!(parsed.drift_records, 1);
        assert_eq!(parsed.drift_matched, 0);
        assert_eq!(
            value["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_SCHEMA
        );
        assert_eq!(value["status_coverage"]["matched"], 1);
        assert_eq!(value["status_coverage"]["drifted"], 1);
        assert_eq!(value["drift_fields"]["records"], 1);
        assert_eq!(value["drift_fields"]["matched"], 0);
        Ok(())
    }

    fn sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json(
    ) -> Result<String, Box<dyn std::error::Error>> {
        let (_, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        Ok(summary_json)
    }

    fn sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs(
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let matched_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &json_line,
            )?;
        let matched_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &matched_report,
            )?;

        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let stale_line = serde_json::to_string(&value)?;
        let drifted_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &stale_line,
            )?;
        let drifted_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &drifted_report,
            )?;
        let log = format!("matched verifier\n{matched_line}\ndrifted verifier\n{drifted_line}\n");

        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
                &log,
            )?;
        let summary_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &summary,
            )?;
        Ok((log, summary_json))
    }

    #[test]
    fn parses_rollup_check_log_summary_json_verification_rollup_verdict_summary_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let log = format!("dashboard summary\n{summary_json}\n");

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &log,
            )?;

        assert_eq!(parsed.records, 2);
        assert_eq!(parsed.matched, 1);
        assert_eq!(parsed.drifted, 1);
        assert_eq!(parsed.drift_records, 1);
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_schema_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["schema"] = json!("locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary.v0");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &value,
            )
            .expect_err("schema drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(_)
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_missing_group(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value
            .as_object_mut()
            .expect("json object")
            .remove("drift_fields");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &value,
            )
            .expect_err("missing group");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("drift_fields")
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_status_coverage_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["status_coverage"]["matched"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &value,
            )
            .expect_err("status coverage drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_coverage.matched",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_drift_field_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary_json =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_json()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line(
                &value,
            )
            .expect_err("drift field drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn verifies_rollup_check_log_summary_json_verification_rollup_verdict_summary_against_source(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");

        let verified =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;

        assert_eq!(verified.records, 2);
        assert_eq!(verified.matched, 1);
        assert_eq!(verified.drifted, 1);
        assert_eq!(verified.drift_records, 1);
        assert_eq!(report.drift, None);
        assert_eq!(report.actual, verified);
        Ok(())
    }

    #[test]
    fn reports_rollup_check_log_summary_json_verification_rollup_verdict_summary_record_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let strict_error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("strict verifier rejects records drift");

        assert_eq!(
            report.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        assert_eq!(report.expected.records, 2);
        assert_eq!(report.actual.records, 1);
        assert!(matches!(
            strict_error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn formats_matched_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;

        assert!(report.is_matched());
        assert_eq!(report.status_str(), "matched");
        assert_eq!(
            value["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_SCHEMA
        );
        assert_eq!(value["status"], "matched");
        assert_eq!(value["matched"], true);
        assert!(value["drift"].is_null());
        assert_eq!(value["expected"]["records"], 2);
        assert_eq!(value["actual"]["records"], 2);
        assert_eq!(value["expected"]["drift_fields"]["records"], 1);
        assert_eq!(value["actual"]["drift_fields"]["records"], 1);
        Ok(())
    }

    #[test]
    fn formats_drifted_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        let strict_error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("strict verifier rejects records drift");

        assert!(!report.is_matched());
        assert_eq!(report.status_str(), "drifted");
        assert_eq!(
            report.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        assert_eq!(value["status"], "drifted");
        assert_eq!(value["matched"], false);
        assert_eq!(value["expected"]["records"], 2);
        assert_eq!(value["actual"]["records"], 1);
        assert_eq!(value["drift"]["field"], "records");
        assert_eq!(value["drift"]["expected"], 2);
        assert_eq!(value["drift"]["actual"], 1);
        assert!(matches!(
            strict_error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn parses_matched_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let verdict_log = format!("published summary verdict\n{verdict_line}\n");

        let parsed_line =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &verdict_line,
            )?;
        let parsed_log =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &verdict_log,
            )?;

        assert_eq!(parsed_line, report);
        assert_eq!(parsed_log, report);
        assert!(parsed_log.is_matched());
        Ok(())
    }

    #[test]
    fn parses_drifted_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &verdict_line,
            )?;

        assert_eq!(parsed, report);
        assert!(!parsed.is_matched());
        assert_eq!(
            parsed.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_status_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["status"] = json!("drifted");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &value,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "status",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_drift_payload_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["drift"]["actual"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &value,
            )
            .expect_err("drift payload drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_nested_group_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["expected"]["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &value,
            )
            .expect_err("nested group drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn summarizes_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdicts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;

        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&json_line)?;

        assert_eq!(rollup.summary.records, 2);
        assert_eq!(rollup.summary.matched, 1);
        assert_eq!(rollup.summary.drifted, 1);
        assert_eq!(rollup.summary.drift_records, 1);
        assert_eq!(
            value["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_SCHEMA
        );
        assert_eq!(value["records"], 2);
        assert_eq!(value["status_coverage"]["matched"], 1);
        assert_eq!(value["status_coverage"]["drifted"], 1);
        assert_eq!(value["drift_fields"]["records"], 1);
        assert_eq!(value["drift_fields"]["matched"], 0);
        Ok(())
    }

    fn sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log(
    ) -> Result<String, Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let summary_log = format!("dashboard summary\n{summary_json}\n");
        let matched_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let matched_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &matched_report,
            )?;

        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;
        let drifted_report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )?;
        let drifted_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line(
                &drifted_report,
            )?;

        Ok(format!(
            "matched summary verdict\n{matched_line}\ndrifted summary verdict\n{drifted_line}\n"
        ))
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_without_records(
    ) {
        let error =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                "dashboard noise\n",
            )
            .expect_err("missing verifier-summary verification JSON records");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line"
            )
        ));
    }

    #[test]
    fn parses_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("published summary verdict rollup\n{json_line}\n");

        let parsed_line =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &json_line,
            )?;
        let parsed_log =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &rollup_log,
            )?;

        assert_eq!(parsed_line, rollup);
        assert_eq!(parsed_log, rollup);
        assert_eq!(parsed_log.summary.records, 2);
        assert_eq!(parsed_log.summary.drift_records, 1);
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_status_group_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["status_coverage"]["matched"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("status coverage drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_coverage.matched",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_drift_field_group_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &value,
            )
            .expect_err("drift fields drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_log_without_records(
    ) {
        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                "dashboard noise\n",
            )
            .expect_err("missing verifier-summary verification rollup JSON");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line"
            )
        ));
    }

    #[test]
    fn verifies_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_against_source(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let rollup_log = format!("dashboard rollup\n{json_line}\n");

        let verified =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;

        assert_eq!(verified.summary.records, 2);
        assert_eq!(verified.summary.matched, 1);
        assert_eq!(verified.summary.drifted, 1);
        assert_eq!(verified.summary.drift_records, 1);
        assert_eq!(report.drift, None);
        assert!(report.is_matched());
        assert_eq!(report.status_str(), "matched");
        Ok(())
    }

    #[test]
    fn reports_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_record_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let rollup_log = serde_json::to_string(&value)?;

        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )?;
        let strict_error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("strict verifier rejects records drift");

        assert!(!report.is_matched());
        assert_eq!(report.status_str(), "drifted");
        assert_eq!(
            report.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        assert_eq!(report.expected.summary.records, 2);
        assert_eq!(report.actual.summary.records, 1);
        assert!(matches!(
            strict_error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn formats_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_match_json(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_json,
            )?;
        let verdict_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_json)?;

        assert_eq!(
            value["schema"],
            json!(REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_LOG_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SUMMARY_VERIFICATION_ROLLUP_VERIFICATION_SCHEMA)
        );
        assert_eq!(value["status"], json!("matched"));
        assert_eq!(value["matched"], json!(true));
        assert!(value["drift"].is_null());
        assert_eq!(value["expected"]["records"], json!(2));
        assert_eq!(value["expected"]["status_coverage"]["matched"], json!(1));
        assert_eq!(value["expected"]["status_coverage"]["drifted"], json!(1));
        assert_eq!(value["actual"]["records"], json!(2));
        assert_eq!(value["actual"]["drift_fields"]["records"], json!(1));
        Ok(())
    }

    #[test]
    fn formats_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_drift_json(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut rollup_value = serde_json::from_str::<serde_json::Value>(&rollup_json)?;
        rollup_value["records"] = json!(1);
        let drifted_rollup_json = serde_json::to_string(&rollup_value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &drifted_rollup_json,
            )?;
        let verdict_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let value = serde_json::from_str::<serde_json::Value>(&verdict_json)?;
        let strict_error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &drifted_rollup_json,
            )
            .expect_err("strict verifier rejects records drift");

        assert_eq!(value["status"], json!("drifted"));
        assert_eq!(value["matched"], json!(false));
        assert_eq!(value["drift"]["field"], json!("records"));
        assert_eq!(value["drift"]["expected"], json!(2));
        assert_eq!(value["drift"]["actual"], json!(1));
        assert_eq!(value["expected"]["records"], json!(2));
        assert_eq!(value["actual"]["records"], json!(1));
        assert_eq!(value["actual"]["drift_fields"]["records"], json!(1));
        assert!(matches!(
            strict_error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn parses_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_match_json(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_json,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let verdict_log = format!("rollup verification verdict\n{verdict_line}\n");

        let parsed_line =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &verdict_line,
            )?;
        let parsed_log =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_log(
                &verdict_log,
            )?;

        assert_eq!(parsed_line, report);
        assert_eq!(parsed_log, report);
        assert!(parsed_log.is_matched());
        Ok(())
    }

    #[test]
    fn parses_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_drift_json(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut rollup_value = serde_json::from_str::<serde_json::Value>(&rollup_json)?;
        rollup_value["records"] = json!(1);
        let drifted_rollup_json = serde_json::to_string(&rollup_value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &drifted_rollup_json,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;

        let parsed =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &verdict_line,
            )?;

        assert_eq!(parsed, report);
        assert!(!parsed.is_matched());
        assert_eq!(
            parsed.drift.as_ref().map(|drift| drift.field),
            Some("records")
        );
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_status_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_json,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["status"] = json!("drifted");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "status",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_drift_payload_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut rollup_value = serde_json::from_str::<serde_json::Value>(&rollup_json)?;
        rollup_value["records"] = json!(1);
        let drifted_rollup_json = serde_json::to_string(&rollup_value)?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &drifted_rollup_json,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["drift"]["actual"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("drift payload drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "drift",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_nested_group_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let rollup_json =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_json,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["expected"]["drift_fields"]["records"] = json!(0);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line(
                &value,
            )
            .expect_err("nested group drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_rollup_group_drift_before_compare(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_verdict_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["drift_fields"]["records"] = json!(0);
        let rollup_log = serde_json::to_string(&value)?;

        let error =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("grouped parser drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_group_drift_before_compare(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source_log, summary_json) =
            sample_rollup_check_log_summary_json_verification_rollup_verdict_summary_inputs()?;
        let mut value = serde_json::from_str::<serde_json::Value>(&summary_json)?;
        value["drift_fields"]["records"] = json!(0);
        let summary_log = serde_json::to_string(&value)?;

        let error =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("grouped parser drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_without_records() {
        let error =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
                "human status line\n{\"schema\":\"other\"}\n",
            )
            .expect_err("missing verifier JSON");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_verification_rollup_verification_json_line"
            )
        ));
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_verdict_summary_status_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let report =
            check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &json_line,
            )?;
        let verdict_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line(
                &report,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&verdict_line)?;
        value["status"] = json!("drifted");
        let log = serde_json::to_string(&value)?;

        let error =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log(
                &log,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::JsonFieldDrift {
                field: "status",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_record_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let rollup_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("records drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_status_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["status_coverage"]["matched"] = json!(0);
        let rollup_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("status coverage drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_coverage.matched",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_verification_rollup_drift_field_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_log_summary_json_verification_log()?;
        let rollup =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
                &source_log,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
                &rollup,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["drift_fields"]["records"] = json!(0);
        let rollup_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
                &source_log,
                &rollup_log,
            )
            .expect_err("drift field drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "drift_fields.records",
                expected: 1,
                actual: 0
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_record_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (_, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["records"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("record drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "records",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_host_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (_, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["bundle_hosts"] = json!(2);
        value["host_coverage"]["bundle_hosts"] = json!(2);
        let summary_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("host drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "bundle_hosts",
                expected: 1,
                actual: 2
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_status_drift_against_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_log = sample_rollup_check_json_source_log()?;
        let (_, json_line) = sample_rollup_check_log_summary_json(&source_log)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["status_valid_bundles"] = json!(1);
        value["status_coverage"]["valid_bundles"] = json!(1);
        let summary_log = serde_json::to_string(&value)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &source_log,
                &summary_log,
            )
            .expect_err("status drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_valid_bundles",
                expected: 2,
                actual: 1
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_missing_rollup_check_log_summary_json_for_recomputed_log(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;

        let error =
            verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
                &first_line,
                "remote_free_service_telemetry_collection_summary_rollup_check_log=ok\n",
            )
            .expect_err("missing summary json");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_log_summary_json_line"
            )
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_schema_drift() -> Result<(), Box<dyn std::error::Error>>
    {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                &first_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["schema"] =
            json!("locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v0");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &value,
            )
            .expect_err("schema drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(_)
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_missing_group(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                &first_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value
            .as_object_mut()
            .expect("json object")
            .remove("host_coverage");
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &value,
            )
            .expect_err("missing group");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField("host_coverage")
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_host_coverage_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                &first_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["host_coverage"]["bundle_hosts"] = json!(99);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &value,
            )
            .expect_err("host coverage drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "host_coverage.bundle_hosts",
                expected: 1,
                actual: 99
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_log_summary_json_status_coverage_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first = sample_rollup_check();
        let first_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&first)?;
        let summary =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                &first_line,
            )?;
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &summary,
            )?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["status_coverage"]["valid_bundles"] = json!(42);
        let value = serde_json::to_string(&value)?;

        let error =
            parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
                &value,
            )
            .expect_err("status coverage drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "status_coverage.valid_bundles",
                expected: 1,
                actual: 42
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_rollup_check_json_log_without_records() {
        let error =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
                "cargo build finished\nno json here\n",
            )
            .expect_err("missing records");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::MissingField(
                "rollup_check_json_line"
            )
        ));
    }

    #[test]
    fn rejects_rollup_check_json_log_schema_drift() -> Result<(), Box<dyn std::error::Error>> {
        let check = sample_rollup_check();
        let json_line =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        let mut value = serde_json::from_str::<serde_json::Value>(&json_line)?;
        value["schema"] =
            json!("locus.remote_free_service.telemetry.collection_summary_rollup_check.v0");
        let log = format!("{}\n", serde_json::to_string(&value)?);

        let error =
            summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(&log)
                .expect_err("schema drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(_)
        ));
        Ok(())
    }

    #[test]
    fn validates_collection_summary_rollup_artifact_with_host_metadata(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        let artifact_text = rollup_json_with_host("valid", 1, 0);
        fs::write(&rollup_path, &artifact_text)?;

        let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
            &rollup_path,
        )?;

        assert_eq!(check.path, rollup_path);
        assert_eq!(
            check.schema,
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(
            check.artifact_bytes,
            u64::try_from(artifact_text.len()).expect("artifact bytes")
        );
        assert_eq!(
            check.artifact_fingerprint,
            rollup_artifact_fingerprint(&artifact_text)
        );
        assert_eq!(check.summaries, 1);
        assert_eq!(check.valid_bundles, 1);
        assert_eq!(check.drifted_summaries, 0);
        assert_eq!(check.missing_artifacts, 0);
        assert_eq!(check.other_failures, 0);
        assert_eq!(check.timing_ranges, 1);
        assert_eq!(check.bundles, 1);
        assert!(check.rollup_host_present);
        assert_eq!(check.bundle_hosts, 1);
        assert_eq!(check.bundle_hosts_missing, 0);
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn accepts_invalid_collection_summary_rollup_host_metadata_as_no_coverage(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        let artifact_text = rollup_json_with_invalid_host_metadata();
        fs::write(&rollup_path, &artifact_text)?;

        let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
            &rollup_path,
        )?;

        assert_eq!(
            check.schema,
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(
            check.artifact_bytes,
            u64::try_from(artifact_text.len()).expect("artifact bytes")
        );
        assert_eq!(
            check.artifact_fingerprint,
            rollup_artifact_fingerprint(&artifact_text)
        );
        assert_eq!(check.valid_bundles, 1);
        assert_eq!(check.drifted_summaries, 0);
        assert_eq!(check.missing_artifacts, 0);
        assert_eq!(check.other_failures, 0);
        assert!(!check.rollup_host_present);
        assert_eq!(check.bundle_hosts, 0);
        assert_eq!(check.bundle_hosts_missing, 1);
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn rejects_unexpected_collection_summary_rollup_schema(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        fs::write(&rollup_path, rollup_json_with_unexpected_schema())?;

        let error =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&rollup_path)
                .expect_err("unexpected schema");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::UnexpectedSchema(schema)
                if schema == "locus.remote_free_service.telemetry.collection_summary_rollup.v1"
        ));
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn writes_collection_summary_rollup_artifact_for_release_check(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup = RemoteFreeServiceTelemetryCollectionSummaryRollup {
            root: dir.clone(),
            host: Some(RemoteFreeServiceTelemetryCollectionSummaryRollupHost {
                os: "linux".to_owned(),
                arch: "x86_64".to_owned(),
                hostname: Some("bench-host-01".to_owned()),
            }),
            summaries: 1,
            valid_bundles: 1,
            drifted_summaries: 0,
            missing_artifacts: 0,
            other_failures: 0,
            timing_ranges: 1,
            bundles: vec![RemoteFreeServiceTelemetryCollectionSummaryRollupBundle {
                summary: "run-1/collection-summary.json".to_owned(),
                run_id: Some("run-1".to_owned()),
                host: Some(RemoteFreeServiceTelemetryCollectionSummaryHost {
                    os: "linux".to_owned(),
                    arch: "x86_64".to_owned(),
                    hostname: Some("bench-host-01".to_owned()),
                }),
                status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid,
                timing_ranges: 1,
            }],
        };

        let rollup_path =
            write_remote_free_service_telemetry_collection_summary_rollup_artifact(&dir, &rollup)?;
        let artifact_text = fs::read_to_string(&rollup_path)?;
        let artifact = serde_json::from_str::<serde_json::Value>(&artifact_text)?;

        assert_eq!(
            artifact["schema"],
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(artifact["bundles"][0]["status"], "valid");
        assert_eq!(artifact["bundles"][0]["host"]["os"], "linux");
        assert_eq!(artifact["bundles"][0]["host"]["arch"], "x86_64");
        assert_eq!(artifact["bundles"][0]["host"]["hostname"], "bench-host-01");
        assert_eq!(artifact["host"]["os"], "linux");
        assert_eq!(artifact["host"]["arch"], "x86_64");
        assert_eq!(artifact["host"]["hostname"], "bench-host-01");

        let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
            &rollup_path,
        )?;

        assert_eq!(check.valid_bundles, 1);
        assert_eq!(
            check.schema,
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(
            check.artifact_bytes,
            u64::try_from(artifact_text.len()).expect("artifact bytes")
        );
        assert_eq!(
            check.artifact_fingerprint,
            rollup_artifact_fingerprint(&artifact_text)
        );
        assert_eq!(check.drifted_summaries, 0);
        assert_eq!(check.missing_artifacts, 0);
        assert_eq!(check.other_failures, 0);
        assert_eq!(check.timing_ranges, 1);
        assert!(check.rollup_host_present);
        assert_eq!(check.bundle_hosts, 1);
        assert_eq!(check.bundle_hosts_missing, 0);
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn rejects_failed_collection_summary_rollup_rows() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        fs::write(&rollup_path, rollup_json_with_valid_and_drifted_host_rows())?;

        let error =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&rollup_path)
                .expect_err("failed rollup rows");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::FailedBundles {
                valid_bundles: 1,
                drifted_summaries: 1,
                missing_artifacts: 0,
                other_failures: 0,
            }
        ));
        assert!(error
            .to_string()
            .contains("valid_bundles=1 drifted_summaries=1"));
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn rejects_collection_summary_rollup_count_drift() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir()?;
        let rollup_path = dir.join("collection-summary-rollup.json");
        fs::write(&rollup_path, rollup_json("valid", 2, 0))?;

        let error =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&rollup_path)
                .expect_err("rollup count drift");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryCollectionSummaryRollupError::CountDrift {
                field: "valid_bundles",
                expected: 2,
                actual: 1,
            }
        ));
        fs::remove_dir_all(dir)?;
        Ok(())
    }
}
