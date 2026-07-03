#![allow(missing_docs)]

use std::{
    env,
    error::Error,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use locus_validate::{
    parse_remote_free_service_telemetry_collection_summary,
    parse_remote_free_service_telemetry_timing_stability_manifest,
    resolve_remote_free_service_telemetry_collection_summary_manifest_path,
    resolve_remote_free_service_telemetry_collection_summary_validation_summary_path,
    summarize_remote_free_service_telemetry_timing_stability,
    validate_remote_free_service_telemetry_collection_summary_rollup_artifact,
    verify_remote_free_service_telemetry_collection_summary_artifacts,
    write_remote_free_service_telemetry_collection_summary_rollup_artifact,
    RemoteFreeServiceTelemetryCollectionSummaryRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupBundle,
    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
    RemoteFreeServiceTelemetryTimingStabilityRun,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_telemetry_summary_validate".to_owned());
    let summary_path = args.next().ok_or_else(|| usage_error(&program))?;
    if summary_path == "--dir" {
        let root = args.next().ok_or_else(|| usage_error(&program))?;
        let write_rollup = match args.next() {
            Some(arg) if arg == "--write-rollup" => true,
            Some(_) => return Err(Box::new(usage_error(&program))),
            None => false,
        };
        if args.next().is_some() {
            return Err(Box::new(usage_error(&program)));
        }
        let root = Path::new(&root);
        let rollup = validate_summary_directory(root)?;
        println!("{rollup}");
        if write_rollup {
            let artifact_path = write_directory_rollup_artifact(root, &rollup)?;
            let byte_count = fs::metadata(&artifact_path)?.len();
            println!(
                "remote_free_service_telemetry_collection_summary_rollup_artifact=written path={} bytes={}",
                artifact_path.display(),
                byte_count
            );
        }
        return Ok(());
    }
    if summary_path == "--rollup" {
        let rollup_path = args.next().ok_or_else(|| usage_error(&program))?;
        if args.next().is_some() {
            return Err(Box::new(usage_error(&program)));
        }
        let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
            Path::new(&rollup_path),
        )?;
        println!("{check}");
        return Ok(());
    }

    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    let summary_path = PathBuf::from(summary_path);
    let report = validate_summary_path(&summary_path)?;

    println!(
        "remote_free_service_telemetry_collection_summary_validation=ok summary={} manifest={} collection_mode={} run_id={} output_count={}",
        report.summary_path.display(),
        report.manifest_path.display(),
        report.collection_mode,
        report.run_id,
        report.output_count
    );
    println!("{}", report.artifact_report);
    println!("{}", report.validation_summary_report);
    print!("{}", report.stability_output);

    Ok(())
}

#[derive(Debug)]
struct BundleValidationReport {
    summary_path: PathBuf,
    manifest_path: PathBuf,
    collection_mode: String,
    run_id: String,
    output_count: usize,
    artifact_report: String,
    validation_summary_report: String,
    stability_output: String,
    timing_ranges: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct DirectoryRollup {
    root: PathBuf,
    summaries: usize,
    valid_bundles: usize,
    drifted_summaries: usize,
    missing_artifacts: usize,
    other_failures: usize,
    timing_ranges: usize,
    bundles: Vec<BundleRollup>,
}

#[derive(Debug, PartialEq, Eq)]
struct BundleRollup {
    summary_path: PathBuf,
    run_id: Option<String>,
    status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
    timing_ranges: usize,
}

impl std::fmt::Display for DirectoryRollup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

fn validate_summary_path(summary_path: &Path) -> Result<BundleValidationReport, Box<dyn Error>> {
    let summary_text = fs::read_to_string(summary_path)?;
    let summary = parse_remote_free_service_telemetry_collection_summary(&summary_text)?;
    let artifact_report =
        verify_remote_free_service_telemetry_collection_summary_artifacts(summary_path, &summary)?;
    let manifest_path = resolve_remote_free_service_telemetry_collection_summary_manifest_path(
        summary_path,
        &summary,
    )?;
    let validation_summary_path =
        resolve_remote_free_service_telemetry_collection_summary_validation_summary_path(
            summary_path,
            &summary,
        )?;
    let stability_output = stability_report_from_manifest(&manifest_path)?;
    let saved_validation_summary = fs::read_to_string(&validation_summary_path)?;
    let validation_summary_report = compare_validation_summary(
        &validation_summary_path,
        &saved_validation_summary,
        &stability_output.text,
    )?;

    Ok(BundleValidationReport {
        summary_path: summary_path.to_path_buf(),
        manifest_path,
        collection_mode: summary.collection_mode,
        run_id: summary.run_id,
        output_count: summary.output_count,
        artifact_report: artifact_report.to_string(),
        validation_summary_report,
        timing_ranges: stability_output.timing_ranges,
        stability_output: stability_output.text,
    })
}

fn validate_summary_directory(root: &Path) -> Result<DirectoryRollup, Box<dyn Error>> {
    let mut summary_paths = Vec::new();
    collect_summary_paths(root, &mut summary_paths)?;
    summary_paths.sort();

    let mut rollup = DirectoryRollup {
        root: root.to_path_buf(),
        summaries: summary_paths.len(),
        ..DirectoryRollup::default()
    };

    for summary_path in summary_paths {
        match validate_summary_path(&summary_path) {
            Ok(report) => {
                rollup.valid_bundles += 1;
                rollup.timing_ranges += report.timing_ranges;
                rollup.bundles.push(BundleRollup {
                    summary_path,
                    run_id: Some(report.run_id),
                    status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid,
                    timing_ranges: report.timing_ranges,
                });
            }
            Err(error) => {
                let message = error.to_string();
                let status = classify_validation_error(&message);
                match status {
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid => {
                        unreachable!("errors cannot validate")
                    }
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::DriftedSummary => {
                        rollup.drifted_summaries += 1;
                    }
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::MissingArtifact => {
                        rollup.missing_artifacts += 1;
                    }
                    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::OtherFailure => {
                        rollup.other_failures += 1;
                    }
                }
                rollup.bundles.push(BundleRollup {
                    run_id: read_summary_run_id(&summary_path).ok(),
                    summary_path,
                    status,
                    timing_ranges: 0,
                });
            }
        }
    }

    Ok(rollup)
}

fn classify_validation_error(
    message: &str,
) -> RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus {
    if message.contains("validation summary drift") {
        RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::DriftedSummary
    } else if message.contains("No such file")
        || message.contains("missing remote-free service telemetry collection summary")
        || message
            .contains("failed to read remote-free service telemetry collection summary artifact")
    {
        RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::MissingArtifact
    } else {
        RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::OtherFailure
    }
}

fn read_summary_run_id(summary_path: &Path) -> Result<String, Box<dyn Error>> {
    let summary_text = fs::read_to_string(summary_path)?;
    let summary = parse_remote_free_service_telemetry_collection_summary(&summary_text)?;
    Ok(summary.run_id)
}

fn write_directory_rollup_artifact(
    root: &Path,
    rollup: &DirectoryRollup,
) -> Result<PathBuf, Box<dyn Error>> {
    let rollup = to_collection_summary_rollup(root, rollup)?;
    Ok(write_remote_free_service_telemetry_collection_summary_rollup_artifact(root, &rollup)?)
}

fn to_collection_summary_rollup(
    root: &Path,
    rollup: &DirectoryRollup,
) -> Result<RemoteFreeServiceTelemetryCollectionSummaryRollup, Box<dyn Error>> {
    let bundles = rollup
        .bundles
        .iter()
        .map(|bundle| {
            let summary_path = bundle
                .summary_path
                .strip_prefix(root)
                .unwrap_or(&bundle.summary_path);
            Ok(RemoteFreeServiceTelemetryCollectionSummaryRollupBundle {
                summary: summary_path.display().to_string(),
                run_id: bundle.run_id.clone(),
                status: bundle.status,
                timing_ranges: u64::try_from(bundle.timing_ranges)?,
            })
        })
        .collect::<Result<Vec<_>, std::num::TryFromIntError>>()?;

    Ok(RemoteFreeServiceTelemetryCollectionSummaryRollup {
        root: rollup.root.clone(),
        summaries: u64::try_from(rollup.summaries)?,
        valid_bundles: u64::try_from(rollup.valid_bundles)?,
        drifted_summaries: u64::try_from(rollup.drifted_summaries)?,
        missing_artifacts: u64::try_from(rollup.missing_artifacts)?,
        other_failures: u64::try_from(rollup.other_failures)?,
        timing_ranges: u64::try_from(rollup.timing_ranges)?,
        bundles,
    })
}

fn collect_summary_paths(root: &Path, summary_paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_summary_paths(&path, summary_paths)?;
        } else if path
            .file_name()
            .is_some_and(|file_name| file_name == "collection-summary.json")
        {
            summary_paths.push(path);
        }
    }
    Ok(())
}

struct StabilityOutput {
    text: String,
    timing_ranges: usize,
}

fn stability_report_from_manifest(
    manifest_path: &Path,
) -> Result<StabilityOutput, Box<dyn std::error::Error>> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest = parse_remote_free_service_telemetry_timing_stability_manifest(&manifest_text)?;
    let baseline_output = fs::read_to_string(resolve_manifest_path(
        manifest_path,
        &manifest.baseline.path,
    ))?;
    let candidate_labels = manifest
        .candidates
        .iter()
        .map(|entry| entry.label.clone())
        .collect::<Vec<_>>();
    let candidate_outputs = manifest
        .candidates
        .iter()
        .map(|entry| fs::read_to_string(resolve_manifest_path(manifest_path, &entry.path)))
        .collect::<Result<Vec<_>, _>>()?;
    let candidate_runs = candidate_labels
        .iter()
        .zip(candidate_outputs.iter())
        .map(
            |(label, output)| RemoteFreeServiceTelemetryTimingStabilityRun {
                label: label.as_str(),
                output: output.as_str(),
            },
        )
        .collect::<Vec<_>>();
    let report = summarize_remote_free_service_telemetry_timing_stability(
        RemoteFreeServiceTelemetryTimingStabilityRun {
            label: manifest.baseline.label.as_str(),
            output: baseline_output.as_str(),
        },
        &candidate_runs,
    )?;

    let mut output = String::new();
    let _ = writeln!(&mut output, "{report}");
    for discard in report.discards {
        let _ = writeln!(&mut output, "{discard}");
    }
    for range in &report.ranges {
        let _ = writeln!(&mut output, "{range}");
    }

    Ok(StabilityOutput {
        timing_ranges: report.ranges.len(),
        text: output,
    })
}

fn compare_validation_summary(
    path: &Path,
    saved: &str,
    computed: &str,
) -> Result<String, io::Error> {
    if saved == computed {
        return Ok(format!(
            "remote_free_service_telemetry_validation_summary=matched path={} bytes={}",
            path.display(),
            saved.len()
        ));
    }

    Err(io::Error::other(format!(
        "remote-free service telemetry validation summary drift: path={} saved_bytes={} computed_bytes={}",
        path.display(),
        saved.len(),
        computed.len()
    )))
}

fn resolve_manifest_path(manifest_path: &Path, entry_path: &str) -> PathBuf {
    let path = PathBuf::from(entry_path);
    if path.is_absolute() {
        path
    } else {
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(path)
    }
}

fn usage_error(program: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "usage: {program} <collection-summary.json>\n       {program} --dir <evidence-root> [--write-rollup]\n       {program} --rollup <collection-summary-rollup.json>"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        compare_validation_summary, validate_summary_directory, write_directory_rollup_artifact,
    };
    use locus_validate::validate_remote_free_service_telemetry_collection_summary_rollup_artifact;
    use serde_json::json;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    const APPLY_CONFIRM_SAMPLE: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample","line":"remote_free_service_runtime_apply_confirm_sample submitted_count=768 final_previous_config_present=false","fields":{"submitted_count":768,"drained_count":768,"released_bytes":3145728,"confirm_count":1,"rollback_count":0,"final_previous_config_present":false}}"#;
    const APPLY_CONFIRM_SUMMARY: &str = r#"{"schema":"locus.remote_free_service.telemetry.sample.v1","benchmark":"remote_free_service_runtime_apply_confirm","sample":"remote_free_service_runtime_apply_confirm_sample_summary","line":"remote_free_service_runtime_apply_confirm_sample_summary samples=8 policy_drains_mean=12.000","fields":{"samples":8,"policy_drains_mean":12.000}}"#;

    fn temp_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!(
            "locus-summary-rollup-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir)?;
        Ok(dir)
    }

    fn timed_output(estimate: &str) -> String {
        format!(
            "{APPLY_CONFIRM_SAMPLE}\n{APPLY_CONFIRM_SUMMARY}\nremote_free_service_runtime_apply_confirm\n                        time:   [56.500 us {estimate} us 57.500 us]\n"
        )
    }

    fn write_bundle(
        root: &Path,
        name: &str,
        drift_summary: bool,
        remove_output: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bundle = root.join(name);
        fs::create_dir(&bundle)?;
        let baseline = timed_output("56.600");
        let candidate = timed_output("57.125");
        fs::write(bundle.join("run-01.txt"), &baseline)?;
        fs::write(bundle.join("run-02.txt"), &candidate)?;
        fs::write(
            bundle.join("manifest.txt"),
            "# role label path\nbaseline run-01 run-01.txt\ncandidate run-02 run-02.txt\n",
        )?;
        let summary = "remote_free_service_telemetry_timing_stability=stable baseline=run-01 candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1\nremote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=56600000 max_estimate_ps=57125000 spread_ps=525000\n";
        fs::write(
            bundle.join("validation-summary.txt"),
            if drift_summary { "drifted\n" } else { summary },
        )?;
        let artifacts = json!([
            {
                "kind": "output",
                "role": "baseline",
                "path": "run-01.txt",
                "byte_count": baseline.len()
            },
            {
                "kind": "output",
                "role": "candidate",
                "path": "run-02.txt",
                "byte_count": candidate.len()
            },
            {
                "kind": "manifest",
                "path": "manifest.txt",
                "byte_count": fs::metadata(bundle.join("manifest.txt"))?.len()
            },
            {
                "kind": "validation_summary",
                "path": "validation-summary.txt",
                "byte_count": fs::metadata(bundle.join("validation-summary.txt"))?.len()
            }
        ]);
        let summary_json = json!({
            "schema": "locus.remote_free_service.telemetry.collection_summary.v1",
            "collection_mode": "saved_output",
            "run_id": name,
            "output_count": 2,
            "criterion_args": [],
            "sources": [
                {
                    "role": "baseline",
                    "label": "run-01",
                    "input": "run-01.txt",
                    "artifact": "run-01.txt"
                },
                {
                    "role": "candidate",
                    "label": "run-02",
                    "input": "run-02.txt",
                    "artifact": "run-02.txt"
                }
            ],
            "artifacts": artifacts
        });
        fs::write(
            bundle.join("collection-summary.json"),
            format!("{}\n", serde_json::to_string_pretty(&summary_json)?),
        )?;
        if remove_output {
            fs::remove_file(bundle.join("run-02.txt"))?;
        }
        Ok(())
    }

    #[test]
    fn reports_matching_validation_summary() {
        let report = compare_validation_summary(
            Path::new("validation-summary.txt"),
            "summary\n",
            "summary\n",
        )
        .expect("matched");

        assert_eq!(
            report,
            "remote_free_service_telemetry_validation_summary=matched path=validation-summary.txt bytes=8"
        );
    }

    #[test]
    fn rejects_drifted_validation_summary() {
        let error =
            compare_validation_summary(Path::new("validation-summary.txt"), "old\n", "new\n")
                .expect_err("drift");

        assert!(error
            .to_string()
            .contains("validation summary drift: path=validation-summary.txt"));
    }

    #[test]
    fn rolls_up_valid_drifted_and_missing_bundles() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir()?;
        write_bundle(&root, "valid", false, false)?;
        write_bundle(&root, "drifted", true, false)?;
        write_bundle(&root, "missing", false, true)?;

        let rollup = validate_summary_directory(&root)?;

        assert_eq!(rollup.summaries, 3);
        assert_eq!(rollup.valid_bundles, 1);
        assert_eq!(rollup.drifted_summaries, 1);
        assert_eq!(rollup.missing_artifacts, 1);
        assert_eq!(rollup.other_failures, 0);
        assert_eq!(rollup.timing_ranges, 1);
        let bundle_rows = rollup
            .bundles
            .iter()
            .map(|bundle| {
                (
                    bundle.run_id.as_deref(),
                    bundle.status.as_str(),
                    bundle.timing_ranges,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bundle_rows,
            vec![
                (Some("drifted"), "drifted_summary", 0),
                (Some("missing"), "missing_artifact", 0),
                (Some("valid"), "valid", 1),
            ]
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn writes_directory_rollup_artifact() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir()?;
        write_bundle(&root, "valid", false, false)?;
        let rollup = validate_summary_directory(&root)?;

        let path = write_directory_rollup_artifact(&root, &rollup)?;
        let artifact = fs::read_to_string(&path)?;
        let artifact = serde_json::from_str::<serde_json::Value>(&artifact)?;

        assert_eq!(
            artifact["schema"],
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(artifact["root"], root.to_string_lossy().as_ref());
        assert_eq!(artifact["summaries"], 1);
        assert_eq!(artifact["valid_bundles"], 1);
        assert_eq!(artifact["drifted_summaries"], 0);
        assert_eq!(artifact["missing_artifacts"], 0);
        assert_eq!(artifact["other_failures"], 0);
        assert_eq!(artifact["timing_ranges"], 1);
        assert_eq!(
            artifact["bundles"].as_array().expect("bundle rows").len(),
            1
        );
        assert_eq!(
            artifact["bundles"][0]["summary"],
            "valid/collection-summary.json"
        );
        assert_eq!(artifact["bundles"][0]["run_id"], "valid");
        assert_eq!(artifact["bundles"][0]["status"], "valid");
        assert_eq!(artifact["bundles"][0]["timing_ranges"], 1);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn validates_clean_rollup_artifact() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir()?;
        write_bundle(&root, "valid", false, false)?;
        let rollup = validate_summary_directory(&root)?;
        let path = write_directory_rollup_artifact(&root, &rollup)?;

        let check =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&path)?;

        assert_eq!(check.path, path);
        assert_eq!(check.summaries, 1);
        assert_eq!(check.valid_bundles, 1);
        assert_eq!(check.timing_ranges, 1);
        assert_eq!(check.bundles, 1);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_failed_rollup_bundle_rows() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir()?;
        write_bundle(&root, "valid", false, false)?;
        let rollup = validate_summary_directory(&root)?;
        let path = write_directory_rollup_artifact(&root, &rollup)?;
        let mut artifact = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&path)?)?;
        artifact["valid_bundles"] = json!(0);
        artifact["drifted_summaries"] = json!(1);
        artifact["bundles"][0]["status"] = json!("drifted_summary");
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&artifact)?),
        )?;

        let error =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&path)
                .expect_err("failed bundle row");

        assert!(error.to_string().contains("rollup contains failed bundles"));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_rollup_count_drift() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir()?;
        write_bundle(&root, "valid", false, false)?;
        let rollup = validate_summary_directory(&root)?;
        let path = write_directory_rollup_artifact(&root, &rollup)?;
        let mut artifact = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&path)?)?;
        artifact["valid_bundles"] = json!(2);
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&artifact)?),
        )?;

        let error =
            validate_remote_free_service_telemetry_collection_summary_rollup_artifact(&path)
                .expect_err("count drift");

        assert!(error
            .to_string()
            .contains("rollup count drift: field=valid_bundles expected=2 actual=1"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
