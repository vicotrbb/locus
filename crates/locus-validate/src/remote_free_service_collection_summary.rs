use std::{
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use serde_json::Value;

/// Expected schema for remote-free service telemetry collection summaries.
pub const REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_SCHEMA: &str =
    "locus.remote_free_service.telemetry.collection_summary.v1";

/// Parsed remote-free service telemetry collection summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceTelemetryCollectionSummary {
    /// Collection mode used by the collector.
    pub collection_mode: String,
    /// Stable run id used as the evidence directory name.
    pub run_id: String,
    /// Number of captured output artifacts.
    pub output_count: usize,
    /// Criterion arguments used for benchmark capture.
    pub criterion_args: Vec<String>,
    /// Source entries indexed by the summary.
    pub sources: Vec<RemoteFreeServiceTelemetryCollectionSummarySource>,
    /// Artifact entries indexed by the summary.
    pub artifacts: Vec<RemoteFreeServiceTelemetryCollectionSummaryArtifact>,
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

impl fmt::Display for RemoteFreeServiceTelemetryCollectionSummaryArtifactReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts={} verified_bytes={}",
            self.verified_artifacts, self.verified_bytes
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
            | Self::InvalidArtifactPath(_)
            | Self::ByteCountMismatch { .. } => None,
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
    let base_dir = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let manifest = summary
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == "manifest")
        .ok_or(RemoteFreeServiceTelemetryCollectionSummaryError::MissingManifestArtifact)?;

    resolve_summary_artifact_path(base_dir, &manifest.path)
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
        parse_remote_free_service_telemetry_collection_summary,
        resolve_remote_free_service_telemetry_collection_summary_manifest_path,
        verify_remote_free_service_telemetry_collection_summary_artifacts,
        RemoteFreeServiceTelemetryCollectionSummaryError,
    };
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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

    fn temp_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let dir = env::temp_dir().join(format!(
            "locus-summary-validate-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&dir)?;
        Ok(dir)
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
        assert_eq!(summary.output_count, 1);
        assert_eq!(summary.criterion_args, ["--sample-size", "10"]);
        assert_eq!(summary.sources[0].artifact, "run-01.txt");
        assert_eq!(summary.artifacts[0].byte_count, 3);
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
}
