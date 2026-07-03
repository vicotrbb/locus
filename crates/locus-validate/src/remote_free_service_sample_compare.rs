use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde_json::Value;

const SAMPLE_SCHEMA: &str = "locus.remote_free_service.telemetry.sample.v1";

/// Stable key for one remote-free service telemetry sample row.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RemoteFreeServiceTelemetrySampleKey {
    /// Criterion benchmark label.
    pub benchmark: String,
    /// Sample row label.
    pub sample: String,
}

/// Parsed remote-free service telemetry JSON sample row.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFreeServiceTelemetrySampleRow {
    /// Stable sample key.
    pub key: RemoteFreeServiceTelemetrySampleKey,
    /// Original text sample row.
    pub line: String,
    /// Parsed row fields.
    pub fields: BTreeMap<String, Value>,
}

/// Comparison status for two remote-free service telemetry sample sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetrySampleCompareStatus {
    /// Every sample and field matched.
    Stable,
    /// One or more samples or fields drifted.
    Drift,
}

impl RemoteFreeServiceTelemetrySampleCompareStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Drift => "drift",
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleCompareStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sample or field drift between two benchmark outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFreeServiceTelemetrySampleDrift {
    /// Criterion benchmark label.
    pub benchmark: String,
    /// Sample row label.
    pub sample: String,
    /// Field name. This is `sample` when the row is missing from one side.
    pub field: String,
    /// Baseline value, or `missing`.
    pub baseline: String,
    /// Candidate value, or `missing`.
    pub candidate: String,
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleDrift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_sample_drift benchmark={} sample={} field={} baseline={} candidate={}",
            self.benchmark, self.sample, self.field, self.baseline, self.candidate
        )
    }
}

/// Comparison report for two remote-free service telemetry sample sets.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFreeServiceTelemetrySampleCompareReport {
    /// Comparison status.
    pub status: RemoteFreeServiceTelemetrySampleCompareStatus,
    /// Number of baseline sample rows.
    pub baseline_samples: usize,
    /// Number of candidate sample rows.
    pub candidate_samples: usize,
    /// Number of sample keys present in both runs.
    pub compared_samples: usize,
    /// Drift entries.
    pub drifts: Vec<RemoteFreeServiceTelemetrySampleDrift>,
}

/// Criterion timing interval for one remote-free service telemetry benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingInterval {
    /// Lower bound in picoseconds.
    pub lower_ps: u128,
    /// Point estimate in picoseconds.
    pub estimate_ps: u128,
    /// Upper bound in picoseconds.
    pub upper_ps: u128,
}

/// Timing delta for one benchmark after counter comparison passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryTimingDelta {
    /// Criterion benchmark label.
    pub benchmark: String,
    /// Baseline point estimate in picoseconds.
    pub baseline_estimate_ps: u128,
    /// Candidate point estimate in picoseconds.
    pub candidate_estimate_ps: u128,
    /// Candidate minus baseline point estimate in picoseconds.
    pub estimate_delta_ps: i128,
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_timing_delta benchmark={} baseline_estimate_ps={} candidate_estimate_ps={} estimate_delta_ps={}",
            self.benchmark,
            self.baseline_estimate_ps,
            self.candidate_estimate_ps,
            self.estimate_delta_ps
        )
    }
}

/// Combined counter and timing comparison status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetrySampleTimingCompareStatus {
    /// Sample counters matched and timing deltas were emitted.
    Stable,
    /// Sample counters drifted, so timing deltas were suppressed.
    CounterDrift,
}

impl RemoteFreeServiceTelemetrySampleTimingCompareStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::CounterDrift => "counter_drift",
        }
    }
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleTimingCompareStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Combined counter and timing comparison report.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFreeServiceTelemetrySampleTimingCompareReport {
    /// Combined comparison status.
    pub status: RemoteFreeServiceTelemetrySampleTimingCompareStatus,
    /// Sample counter comparison report.
    pub samples: RemoteFreeServiceTelemetrySampleCompareReport,
    /// Timing deltas. Empty when counters drift.
    pub timing_deltas: Vec<RemoteFreeServiceTelemetryTimingDelta>,
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleTimingCompareReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_sample_timing_compare={} baseline_samples={} candidate_samples={} compared_samples={} drift_entries={} timing_entries={}",
            self.status,
            self.samples.baseline_samples,
            self.samples.candidate_samples,
            self.samples.compared_samples,
            self.samples.drifts.len(),
            self.timing_deltas.len()
        )
    }
}

impl RemoteFreeServiceTelemetrySampleCompareReport {
    /// Returns true when no sample or field drift was found.
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.status == RemoteFreeServiceTelemetrySampleCompareStatus::Stable
    }
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleCompareReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_sample_compare={} baseline_samples={} candidate_samples={} compared_samples={} drift_entries={}",
            self.status,
            self.baseline_samples,
            self.candidate_samples,
            self.compared_samples,
            self.drifts.len()
        )
    }
}

/// Error returned when parsing remote-free service telemetry JSON sample rows.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetrySampleParseError {
    /// No JSON sample rows were found.
    Empty,
    /// A JSON line was malformed.
    Json {
        /// One-based line number.
        line: usize,
        /// JSON parse error.
        source: serde_json::Error,
    },
    /// A JSON row did not contain the expected object shape.
    InvalidRow {
        /// One-based line number.
        line: usize,
        /// Parse failure reason.
        reason: &'static str,
    },
    /// A row used an unexpected schema.
    UnexpectedSchema {
        /// One-based line number.
        line: usize,
        /// Schema value.
        schema: String,
    },
    /// A benchmark and sample pair appeared more than once.
    DuplicateSample {
        /// Criterion benchmark label.
        benchmark: String,
        /// Sample row label.
        sample: String,
    },
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("missing remote-free service telemetry JSON sample rows"),
            Self::Json { line, source } => {
                write!(
                    f,
                    "invalid remote-free service telemetry JSON on line {line}: {source}"
                )
            }
            Self::InvalidRow { line, reason } => {
                write!(
                    f,
                    "invalid remote-free service telemetry JSON row on line {line}: {reason}"
                )
            }
            Self::UnexpectedSchema { line, schema } => {
                write!(
                    f,
                    "unexpected remote-free service telemetry schema on line {line}: {schema}"
                )
            }
            Self::DuplicateSample { benchmark, sample } => {
                write!(
                    f,
                    "duplicate remote-free service telemetry sample row: benchmark={benchmark} sample={sample}"
                )
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetrySampleParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json { source, .. } => Some(source),
            Self::Empty
            | Self::InvalidRow { .. }
            | Self::UnexpectedSchema { .. }
            | Self::DuplicateSample { .. } => None,
        }
    }
}

/// Error returned when parsing remote-free service telemetry Criterion timings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeServiceTelemetryTimingParseError {
    /// A required benchmark timing interval is missing.
    MissingBenchmark(String),
    /// A benchmark timing interval appeared more than once.
    DuplicateBenchmark(String),
    /// A timing line did not contain a bracketed interval.
    MissingInterval(String),
    /// A timing interval had an unexpected field count.
    InvalidInterval(String),
    /// A timing value was malformed.
    InvalidValue(String),
    /// A timing unit is not supported.
    UnknownUnit(String),
    /// A numeric conversion overflowed.
    Overflow(String),
    /// A timing delta overflowed signed representation.
    DeltaOverflow(String),
}

impl fmt::Display for RemoteFreeServiceTelemetryTimingParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBenchmark(benchmark) => {
                write!(
                    f,
                    "missing remote-free service telemetry timing: {benchmark}"
                )
            }
            Self::DuplicateBenchmark(benchmark) => {
                write!(
                    f,
                    "duplicate remote-free service telemetry timing: {benchmark}"
                )
            }
            Self::MissingInterval(line) => {
                write!(f, "missing Criterion timing interval: {line}")
            }
            Self::InvalidInterval(line) => {
                write!(f, "invalid Criterion timing interval: {line}")
            }
            Self::InvalidValue(value) => {
                write!(f, "invalid Criterion timing value: {value}")
            }
            Self::UnknownUnit(unit) => {
                write!(f, "unknown Criterion timing unit: {unit}")
            }
            Self::Overflow(value) => {
                write!(f, "Criterion timing value overflowed: {value}")
            }
            Self::DeltaOverflow(benchmark) => {
                write!(
                    f,
                    "remote-free service telemetry timing delta overflowed: {benchmark}"
                )
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetryTimingParseError {}

/// Error returned when comparing remote-free service telemetry samples and
/// timings.
#[derive(Debug)]
pub enum RemoteFreeServiceTelemetrySampleTimingCompareError {
    /// Sample row parsing failed.
    Samples(RemoteFreeServiceTelemetrySampleParseError),
    /// Criterion timing parsing failed.
    Timings(RemoteFreeServiceTelemetryTimingParseError),
}

impl fmt::Display for RemoteFreeServiceTelemetrySampleTimingCompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Samples(source) => write!(
                f,
                "invalid remote-free service telemetry sample comparison: {source}"
            ),
            Self::Timings(source) => write!(
                f,
                "invalid remote-free service telemetry timing comparison: {source}"
            ),
        }
    }
}

impl std::error::Error for RemoteFreeServiceTelemetrySampleTimingCompareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Samples(source) => Some(source),
            Self::Timings(source) => Some(source),
        }
    }
}

/// Parses remote-free service telemetry JSON sample rows from benchmark output.
///
/// # Errors
///
/// Returns an error when no JSON sample rows are present, when a JSON row is
/// malformed, or when a sample key appears more than once.
pub fn parse_remote_free_service_telemetry_sample_rows(
    output: &str,
) -> Result<
    BTreeMap<RemoteFreeServiceTelemetrySampleKey, RemoteFreeServiceTelemetrySampleRow>,
    RemoteFreeServiceTelemetrySampleParseError,
> {
    let mut rows = BTreeMap::new();

    for (line_index, line) in output.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }

        let value = serde_json::from_str::<Value>(line).map_err(|source| {
            RemoteFreeServiceTelemetrySampleParseError::Json {
                line: line_number,
                source,
            }
        })?;
        let row = parse_sample_row_value(line_number, &value)?;
        let key = row.key.clone();
        if rows.insert(key.clone(), row).is_some() {
            return Err(
                RemoteFreeServiceTelemetrySampleParseError::DuplicateSample {
                    benchmark: key.benchmark,
                    sample: key.sample,
                },
            );
        }
    }

    if rows.is_empty() {
        return Err(RemoteFreeServiceTelemetrySampleParseError::Empty);
    }

    Ok(rows)
}

/// Compares remote-free service telemetry JSON sample rows from two outputs.
///
/// # Errors
///
/// Returns an error when either output cannot be parsed into unique sample
/// rows.
pub fn compare_remote_free_service_telemetry_sample_outputs(
    baseline_output: &str,
    candidate_output: &str,
) -> Result<RemoteFreeServiceTelemetrySampleCompareReport, RemoteFreeServiceTelemetrySampleParseError>
{
    let baseline = parse_remote_free_service_telemetry_sample_rows(baseline_output)?;
    let candidate = parse_remote_free_service_telemetry_sample_rows(candidate_output)?;
    Ok(compare_remote_free_service_telemetry_samples(
        &baseline, &candidate,
    ))
}

/// Compares remote-free service telemetry JSON sample rows and Criterion
/// timing intervals from two saved benchmark outputs.
///
/// # Errors
///
/// Returns an error when sample rows cannot be parsed or when required timing
/// intervals are missing, duplicated, or malformed. Timing intervals are only
/// required when sample counters are stable.
pub fn compare_remote_free_service_telemetry_sample_outputs_with_timings(
    baseline_output: &str,
    candidate_output: &str,
) -> Result<
    RemoteFreeServiceTelemetrySampleTimingCompareReport,
    RemoteFreeServiceTelemetrySampleTimingCompareError,
> {
    let baseline = parse_remote_free_service_telemetry_sample_rows(baseline_output)
        .map_err(RemoteFreeServiceTelemetrySampleTimingCompareError::Samples)?;
    let candidate = parse_remote_free_service_telemetry_sample_rows(candidate_output)
        .map_err(RemoteFreeServiceTelemetrySampleTimingCompareError::Samples)?;
    let samples = compare_remote_free_service_telemetry_samples(&baseline, &candidate);

    if !samples.is_stable() {
        return Ok(RemoteFreeServiceTelemetrySampleTimingCompareReport {
            status: RemoteFreeServiceTelemetrySampleTimingCompareStatus::CounterDrift,
            samples,
            timing_deltas: Vec::new(),
        });
    }

    let benchmarks = sample_benchmarks(&baseline);
    let baseline_timings =
        parse_remote_free_service_telemetry_timings(baseline_output, &benchmarks)
            .map_err(RemoteFreeServiceTelemetrySampleTimingCompareError::Timings)?;
    let candidate_timings =
        parse_remote_free_service_telemetry_timings(candidate_output, &benchmarks)
            .map_err(RemoteFreeServiceTelemetrySampleTimingCompareError::Timings)?;
    let timing_deltas = compare_timing_intervals(&baseline_timings, &candidate_timings)
        .map_err(RemoteFreeServiceTelemetrySampleTimingCompareError::Timings)?;

    Ok(RemoteFreeServiceTelemetrySampleTimingCompareReport {
        status: RemoteFreeServiceTelemetrySampleTimingCompareStatus::Stable,
        samples,
        timing_deltas,
    })
}

/// Parses Criterion timing intervals for remote-free service telemetry
/// benchmark labels.
///
/// # Errors
///
/// Returns an error when a required benchmark timing interval is missing,
/// duplicated, or malformed.
pub fn parse_remote_free_service_telemetry_timings(
    output: &str,
    benchmarks: &BTreeSet<String>,
) -> Result<
    BTreeMap<String, RemoteFreeServiceTelemetryTimingInterval>,
    RemoteFreeServiceTelemetryTimingParseError,
> {
    let mut timings = BTreeMap::new();
    let mut current = None;

    for line in output.lines().map(str::trim) {
        if benchmarks.contains(line) {
            current = Some(line.to_owned());
            continue;
        }

        let Some(benchmark) = current.as_ref() else {
            continue;
        };
        if !line.starts_with("time:") {
            continue;
        }

        let interval = parse_criterion_timing_interval(line)?;
        if timings.insert(benchmark.clone(), interval).is_some() {
            return Err(
                RemoteFreeServiceTelemetryTimingParseError::DuplicateBenchmark(benchmark.clone()),
            );
        }
        current = None;
    }

    for benchmark in benchmarks {
        if !timings.contains_key(benchmark) {
            return Err(
                RemoteFreeServiceTelemetryTimingParseError::MissingBenchmark(benchmark.clone()),
            );
        }
    }

    Ok(timings)
}

fn parse_sample_row_value(
    line: usize,
    value: &Value,
) -> Result<RemoteFreeServiceTelemetrySampleRow, RemoteFreeServiceTelemetrySampleParseError> {
    let object =
        value
            .as_object()
            .ok_or(RemoteFreeServiceTelemetrySampleParseError::InvalidRow {
                line,
                reason: "row is not an object",
            })?;
    let schema = string_field(line, object, "schema")?;
    if schema != SAMPLE_SCHEMA {
        return Err(
            RemoteFreeServiceTelemetrySampleParseError::UnexpectedSchema {
                line,
                schema: schema.to_owned(),
            },
        );
    }

    let benchmark = string_field(line, object, "benchmark")?.to_owned();
    let sample = string_field(line, object, "sample")?.to_owned();
    let line_text = string_field(line, object, "line")?.to_owned();
    let fields = object
        .get("fields")
        .and_then(Value::as_object)
        .ok_or(RemoteFreeServiceTelemetrySampleParseError::InvalidRow {
            line,
            reason: "fields is not an object",
        })?
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();

    Ok(RemoteFreeServiceTelemetrySampleRow {
        key: RemoteFreeServiceTelemetrySampleKey { benchmark, sample },
        line: line_text,
        fields,
    })
}

fn string_field<'a>(
    line: usize,
    object: &'a serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<&'a str, RemoteFreeServiceTelemetrySampleParseError> {
    object.get(field).and_then(Value::as_str).ok_or(
        RemoteFreeServiceTelemetrySampleParseError::InvalidRow {
            line,
            reason: field,
        },
    )
}

fn compare_remote_free_service_telemetry_samples(
    baseline: &BTreeMap<RemoteFreeServiceTelemetrySampleKey, RemoteFreeServiceTelemetrySampleRow>,
    candidate: &BTreeMap<RemoteFreeServiceTelemetrySampleKey, RemoteFreeServiceTelemetrySampleRow>,
) -> RemoteFreeServiceTelemetrySampleCompareReport {
    let mut drifts = Vec::new();
    let mut compared_samples = 0;

    for (key, baseline_row) in baseline {
        let Some(candidate_row) = candidate.get(key) else {
            drifts.push(missing_drift(key, "present", "missing"));
            continue;
        };
        compared_samples += 1;
        compare_fields(
            key,
            &baseline_row.fields,
            &candidate_row.fields,
            &mut drifts,
        );
    }

    for key in candidate.keys() {
        if !baseline.contains_key(key) {
            drifts.push(missing_drift(key, "missing", "present"));
        }
    }

    RemoteFreeServiceTelemetrySampleCompareReport {
        status: if drifts.is_empty() {
            RemoteFreeServiceTelemetrySampleCompareStatus::Stable
        } else {
            RemoteFreeServiceTelemetrySampleCompareStatus::Drift
        },
        baseline_samples: baseline.len(),
        candidate_samples: candidate.len(),
        compared_samples,
        drifts,
    }
}

fn sample_benchmarks(
    rows: &BTreeMap<RemoteFreeServiceTelemetrySampleKey, RemoteFreeServiceTelemetrySampleRow>,
) -> BTreeSet<String> {
    rows.keys()
        .map(|key| key.benchmark.clone())
        .collect::<BTreeSet<_>>()
}

fn parse_criterion_timing_interval(
    line: &str,
) -> Result<RemoteFreeServiceTelemetryTimingInterval, RemoteFreeServiceTelemetryTimingParseError> {
    let start = line.find('[').ok_or_else(|| {
        RemoteFreeServiceTelemetryTimingParseError::MissingInterval(line.to_owned())
    })?;
    let end = line.find(']').ok_or_else(|| {
        RemoteFreeServiceTelemetryTimingParseError::MissingInterval(line.to_owned())
    })?;
    if end <= start {
        return Err(RemoteFreeServiceTelemetryTimingParseError::InvalidInterval(
            line.to_owned(),
        ));
    }

    let tokens = line[start + 1..end].split_whitespace().collect::<Vec<_>>();
    if tokens.len() != 6 {
        return Err(RemoteFreeServiceTelemetryTimingParseError::InvalidInterval(
            line.to_owned(),
        ));
    }

    Ok(RemoteFreeServiceTelemetryTimingInterval {
        lower_ps: parse_timing_value_ps(tokens[0], tokens[1])?,
        estimate_ps: parse_timing_value_ps(tokens[2], tokens[3])?,
        upper_ps: parse_timing_value_ps(tokens[4], tokens[5])?,
    })
}

fn parse_timing_value_ps(
    value: &str,
    unit: &str,
) -> Result<u128, RemoteFreeServiceTelemetryTimingParseError> {
    let scale_ps = match unit {
        "ps" => 1,
        "ns" => 1_000,
        "us" | "\u{00b5}s" => 1_000_000,
        "ms" => 1_000_000_000,
        "s" => 1_000_000_000_000,
        _ => {
            return Err(RemoteFreeServiceTelemetryTimingParseError::UnknownUnit(
                unit.to_owned(),
            ));
        }
    };

    parse_decimal_scaled(value, scale_ps)
}

fn parse_decimal_scaled(
    value: &str,
    scale: u128,
) -> Result<u128, RemoteFreeServiceTelemetryTimingParseError> {
    let (whole, fractional) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.chars().all(|value| value.is_ascii_digit())
        || !fractional.chars().all(|value| value.is_ascii_digit())
    {
        return Err(RemoteFreeServiceTelemetryTimingParseError::InvalidValue(
            value.to_owned(),
        ));
    }

    let whole = whole
        .parse::<u128>()
        .map_err(|_| RemoteFreeServiceTelemetryTimingParseError::InvalidValue(value.to_owned()))?;
    let whole_scaled = whole
        .checked_mul(scale)
        .ok_or_else(|| RemoteFreeServiceTelemetryTimingParseError::Overflow(value.to_owned()))?;

    if fractional.is_empty() {
        return Ok(whole_scaled);
    }

    let fractional_value = fractional
        .parse::<u128>()
        .map_err(|_| RemoteFreeServiceTelemetryTimingParseError::InvalidValue(value.to_owned()))?;
    let divisor = 10_u128
        .checked_pow(
            u32::try_from(fractional.len()).map_err(|_| {
                RemoteFreeServiceTelemetryTimingParseError::Overflow(value.to_owned())
            })?,
        )
        .ok_or_else(|| RemoteFreeServiceTelemetryTimingParseError::Overflow(value.to_owned()))?;
    let fractional_scaled = fractional_value
        .checked_mul(scale)
        .ok_or_else(|| RemoteFreeServiceTelemetryTimingParseError::Overflow(value.to_owned()))?
        / divisor;

    whole_scaled
        .checked_add(fractional_scaled)
        .ok_or_else(|| RemoteFreeServiceTelemetryTimingParseError::Overflow(value.to_owned()))
}

fn compare_timing_intervals(
    baseline: &BTreeMap<String, RemoteFreeServiceTelemetryTimingInterval>,
    candidate: &BTreeMap<String, RemoteFreeServiceTelemetryTimingInterval>,
) -> Result<Vec<RemoteFreeServiceTelemetryTimingDelta>, RemoteFreeServiceTelemetryTimingParseError>
{
    let mut deltas = Vec::new();
    for (benchmark, baseline_interval) in baseline {
        let candidate_interval = candidate.get(benchmark).ok_or_else(|| {
            RemoteFreeServiceTelemetryTimingParseError::MissingBenchmark(benchmark.clone())
        })?;
        let baseline_estimate = i128::try_from(baseline_interval.estimate_ps).map_err(|_| {
            RemoteFreeServiceTelemetryTimingParseError::DeltaOverflow(benchmark.clone())
        })?;
        let candidate_estimate = i128::try_from(candidate_interval.estimate_ps).map_err(|_| {
            RemoteFreeServiceTelemetryTimingParseError::DeltaOverflow(benchmark.clone())
        })?;
        let estimate_delta_ps = candidate_estimate
            .checked_sub(baseline_estimate)
            .ok_or_else(|| {
                RemoteFreeServiceTelemetryTimingParseError::DeltaOverflow(benchmark.clone())
            })?;

        deltas.push(RemoteFreeServiceTelemetryTimingDelta {
            benchmark: benchmark.clone(),
            baseline_estimate_ps: baseline_interval.estimate_ps,
            candidate_estimate_ps: candidate_interval.estimate_ps,
            estimate_delta_ps,
        });
    }

    Ok(deltas)
}

fn compare_fields(
    key: &RemoteFreeServiceTelemetrySampleKey,
    baseline: &BTreeMap<String, Value>,
    candidate: &BTreeMap<String, Value>,
    drifts: &mut Vec<RemoteFreeServiceTelemetrySampleDrift>,
) {
    for (field, baseline_value) in baseline {
        match candidate.get(field) {
            Some(candidate_value) if candidate_value == baseline_value => {}
            Some(candidate_value) => drifts.push(field_drift(
                key,
                field,
                value_label(baseline_value),
                value_label(candidate_value),
            )),
            None => drifts.push(field_drift(
                key,
                field,
                value_label(baseline_value),
                "missing",
            )),
        }
    }

    for (field, candidate_value) in candidate {
        if !baseline.contains_key(field) {
            drifts.push(field_drift(
                key,
                field,
                "missing",
                value_label(candidate_value),
            ));
        }
    }
}

fn missing_drift(
    key: &RemoteFreeServiceTelemetrySampleKey,
    baseline: &'static str,
    candidate: &'static str,
) -> RemoteFreeServiceTelemetrySampleDrift {
    field_drift(key, "sample", baseline, candidate)
}

fn field_drift(
    key: &RemoteFreeServiceTelemetrySampleKey,
    field: &str,
    baseline: impl Into<String>,
    candidate: impl Into<String>,
) -> RemoteFreeServiceTelemetrySampleDrift {
    RemoteFreeServiceTelemetrySampleDrift {
        benchmark: key.benchmark.clone(),
        sample: key.sample.clone(),
        field: field.to_owned(),
        baseline: baseline.into(),
        candidate: candidate.into(),
    }
}

fn value_label(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_owned(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_remote_free_service_telemetry_sample_outputs,
        compare_remote_free_service_telemetry_sample_outputs_with_timings,
        parse_remote_free_service_telemetry_sample_rows,
        parse_remote_free_service_telemetry_timings, RemoteFreeServiceTelemetrySampleCompareStatus,
        RemoteFreeServiceTelemetrySampleParseError,
        RemoteFreeServiceTelemetrySampleTimingCompareStatus,
        RemoteFreeServiceTelemetryTimingParseError,
    };
    use std::collections::BTreeSet;

    const APPLY_CONFIRM_SAMPLE: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample","line":"remote_free_service_runtime_apply_confirm_sample submitted_count=768 final_previous_config_present=false","fields":{"submitted_count":768,"drained_count":768,"released_bytes":3145728,"confirm_count":1,"rollback_count":0,"final_previous_config_present":false}}"#;
    const APPLY_CONFIRM_SUMMARY: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample_summary","line":"remote_free_service_runtime_apply_confirm_sample_summary samples=8 policy_drains_mean=12.000","fields":{"samples":8,"policy_drains_mean":12.000}}"#;

    fn sample_output() -> String {
        format!("{APPLY_CONFIRM_SAMPLE}\n{APPLY_CONFIRM_SUMMARY}\n")
    }

    fn timed_output(estimate: &str) -> String {
        format!(
            "{}remote_free_service_runtime_apply_confirm\n                        time:   [56.500 us {estimate} us 56.700 us]\n",
            sample_output()
        )
    }

    #[test]
    fn parses_json_sample_rows() {
        let rows = parse_remote_free_service_telemetry_sample_rows(&sample_output()).expect("rows");

        assert_eq!(rows.len(), 2);
        let sample = rows
            .values()
            .find(|row| row.key.sample == "remote_free_service_runtime_apply_confirm_sample")
            .expect("sample");
        assert_eq!(
            sample
                .fields
                .get("submitted_count")
                .expect("submitted count"),
            &serde_json::json!(768)
        );
        assert_eq!(
            sample
                .fields
                .get("final_previous_config_present")
                .expect("previous config flag"),
            &serde_json::json!(false)
        );
    }

    #[test]
    fn compares_matching_outputs_as_stable() {
        let report = compare_remote_free_service_telemetry_sample_outputs(
            &sample_output(),
            &sample_output(),
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetrySampleCompareStatus::Stable
        );
        assert!(report.is_stable());
        assert_eq!(report.baseline_samples, 2);
        assert_eq!(report.candidate_samples, 2);
        assert_eq!(report.compared_samples, 2);
        assert!(report.drifts.is_empty());
        assert_eq!(
            report.to_string(),
            "remote_free_service_telemetry_sample_compare=stable baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=0"
        );
    }

    #[test]
    fn reports_field_drift() {
        let candidate =
            sample_output().replace("\"submitted_count\":768", "\"submitted_count\":769");
        let report =
            compare_remote_free_service_telemetry_sample_outputs(&sample_output(), &candidate)
                .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetrySampleCompareStatus::Drift
        );
        assert_eq!(report.drifts.len(), 1);
        assert_eq!(report.drifts[0].field, "submitted_count");
        assert_eq!(report.drifts[0].baseline, "768");
        assert_eq!(report.drifts[0].candidate, "769");
        assert_eq!(
            report.drifts[0].to_string(),
            "remote_free_service_telemetry_sample_drift benchmark=remote_free_service_runtime_apply_confirm sample=remote_free_service_runtime_apply_confirm_sample field=submitted_count baseline=768 candidate=769"
        );
    }

    #[test]
    fn reports_missing_sample_drift() {
        let report = compare_remote_free_service_telemetry_sample_outputs(
            &sample_output(),
            APPLY_CONFIRM_SAMPLE,
        )
        .expect("report");

        assert_eq!(report.drifts.len(), 1);
        assert_eq!(report.drifts[0].field, "sample");
        assert_eq!(report.drifts[0].baseline, "present");
        assert_eq!(report.drifts[0].candidate, "missing");
    }

    #[test]
    fn rejects_duplicate_sample_keys() {
        let output = format!("{APPLY_CONFIRM_SAMPLE}\n{APPLY_CONFIRM_SAMPLE}\n");
        let error =
            parse_remote_free_service_telemetry_sample_rows(&output).expect_err("duplicate");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetrySampleParseError::DuplicateSample { .. }
        ));
    }

    #[test]
    fn rejects_missing_json_rows() {
        let error =
            parse_remote_free_service_telemetry_sample_rows("only text\n").expect_err("empty");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetrySampleParseError::Empty
        ));
    }

    #[test]
    fn rejects_unexpected_schema() {
        let output = APPLY_CONFIRM_SAMPLE.replace(
            "locus.remote_free_service.telemetry.sample.v1",
            "other.schema",
        );
        let error = parse_remote_free_service_telemetry_sample_rows(&output).expect_err("schema");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetrySampleParseError::UnexpectedSchema { .. }
        ));
    }

    #[test]
    fn parses_criterion_timings_for_sample_benchmarks() {
        let benchmarks = BTreeSet::from(["remote_free_service_runtime_apply_confirm".to_owned()]);
        let timings =
            parse_remote_free_service_telemetry_timings(&timed_output("56.600"), &benchmarks)
                .expect("timings");

        let interval = timings
            .get("remote_free_service_runtime_apply_confirm")
            .expect("interval");
        assert_eq!(interval.lower_ps, 56_500_000);
        assert_eq!(interval.estimate_ps, 56_600_000);
        assert_eq!(interval.upper_ps, 56_700_000);
    }

    #[test]
    fn compares_stable_samples_with_timing_delta() {
        let baseline = timed_output("56.600");
        let candidate = timed_output("57.125");
        let report = compare_remote_free_service_telemetry_sample_outputs_with_timings(
            &baseline, &candidate,
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetrySampleTimingCompareStatus::Stable
        );
        assert_eq!(report.samples.drifts.len(), 0);
        assert_eq!(report.timing_deltas.len(), 1);
        assert_eq!(report.timing_deltas[0].baseline_estimate_ps, 56_600_000);
        assert_eq!(report.timing_deltas[0].candidate_estimate_ps, 57_125_000);
        assert_eq!(report.timing_deltas[0].estimate_delta_ps, 525_000);
        assert_eq!(
            report.to_string(),
            "remote_free_service_telemetry_sample_timing_compare=stable baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=0 timing_entries=1"
        );
        assert_eq!(
            report.timing_deltas[0].to_string(),
            "remote_free_service_telemetry_timing_delta benchmark=remote_free_service_runtime_apply_confirm baseline_estimate_ps=56600000 candidate_estimate_ps=57125000 estimate_delta_ps=525000"
        );
    }

    #[test]
    fn suppresses_timing_delta_when_counters_drift() {
        let baseline = timed_output("56.600");
        let candidate =
            timed_output("57.125").replace("\"submitted_count\":768", "\"submitted_count\":769");
        let report = compare_remote_free_service_telemetry_sample_outputs_with_timings(
            &baseline, &candidate,
        )
        .expect("report");

        assert_eq!(
            report.status,
            RemoteFreeServiceTelemetrySampleTimingCompareStatus::CounterDrift
        );
        assert_eq!(report.samples.drifts.len(), 1);
        assert!(report.timing_deltas.is_empty());
        assert_eq!(
            report.to_string(),
            "remote_free_service_telemetry_sample_timing_compare=counter_drift baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=1 timing_entries=0"
        );
    }

    #[test]
    fn rejects_missing_timing_for_stable_samples() {
        let error = compare_remote_free_service_telemetry_sample_outputs_with_timings(
            &sample_output(),
            &sample_output(),
        )
        .expect_err("missing timing");

        assert!(matches!(
            error,
            super::RemoteFreeServiceTelemetrySampleTimingCompareError::Timings(
                RemoteFreeServiceTelemetryTimingParseError::MissingBenchmark(_)
            )
        ));
    }

    #[test]
    fn rejects_unknown_timing_unit() {
        let benchmarks = BTreeSet::from(["remote_free_service_runtime_apply_confirm".to_owned()]);
        let output = format!(
            "{}remote_free_service_runtime_apply_confirm\n                        time:   [1 zz 2 zz 3 zz]\n",
            sample_output()
        );
        let error =
            parse_remote_free_service_telemetry_timings(&output, &benchmarks).expect_err("unit");

        assert!(matches!(
            error,
            RemoteFreeServiceTelemetryTimingParseError::UnknownUnit(_)
        ));
    }
}
