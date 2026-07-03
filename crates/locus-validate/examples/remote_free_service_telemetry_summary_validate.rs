#![allow(missing_docs)]

use std::{
    env,
    error::Error,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use locus_validate::{
    build_remote_free_service_telemetry_collection_summary_directory_rollup,
    check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
    format_remote_free_service_telemetry_collection_summary_rollup_check_json_line,
    format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line,
    format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line,
    format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line,
    parse_remote_free_service_telemetry_collection_summary,
    parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line,
    parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
    parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log,
    parse_remote_free_service_telemetry_timing_stability_manifest,
    resolve_remote_free_service_telemetry_collection_summary_manifest_path,
    resolve_remote_free_service_telemetry_collection_summary_validation_summary_path,
    summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log,
    summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log,
    summarize_remote_free_service_telemetry_timing_stability,
    validate_remote_free_service_telemetry_collection_summary_rollup_artifact,
    verify_remote_free_service_telemetry_collection_summary_artifacts,
    verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log,
    verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log,
    write_remote_free_service_telemetry_collection_summary_rollup_artifact,
    RemoteFreeServiceTelemetryCollectionSummaryBundleValidation,
    RemoteFreeServiceTelemetryCollectionSummaryHost,
    RemoteFreeServiceTelemetryCollectionSummaryRollup,
    RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus,
    RemoteFreeServiceTelemetryCollectionSummaryRollupHost,
    RemoteFreeServiceTelemetryTimingStabilityRun,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_telemetry_summary_validate".to_owned());
    let summary_path = args.next().ok_or_else(|| usage_error(&program))?;

    if run_mode(&program, &summary_path, &mut args)? {
        return Ok(());
    }

    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    validate_and_print_summary_path(&summary_path)
}

fn run_mode(
    program: &str,
    mode: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match mode {
        "--dir" => run_dir_mode(program, args)?,
        "--rollup" => run_rollup_mode(program, args)?,
        "--rollup-check-json" => run_rollup_check_json_mode(program, args)?,
        "--rollup-check-json-summary" => run_rollup_check_json_summary_mode(program, args)?,
        "--rollup-check-json-summary-verify" => {
            run_rollup_check_json_summary_verify_mode(program, args)?;
        }
        "--rollup-check-json-summary-verify-against" => {
            run_rollup_check_json_summary_verify_against_mode(program, args)?;
        }
        "--rollup-check-json-summary-verify-against-json" => {
            run_rollup_check_json_summary_verify_against_json_mode(program, args)?;
        }
        "--rollup-check-json-summary-verdict-rollup" => {
            run_rollup_check_json_summary_verdict_rollup_mode(program, args)?;
        }
        "--rollup-check-json-summary-verdict-rollup-verify" => {
            run_rollup_check_json_summary_verdict_rollup_verify_mode(program, args)?;
        }
        "--rollup-check-json-summary-verdict-rollup-verify-against" => {
            run_rollup_check_json_summary_verdict_rollup_verify_against_mode(program, args)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn run_dir_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = args.next().ok_or_else(|| usage_error(program))?;
    let write_rollup = match args.next() {
        Some(arg) if arg == "--write-rollup" => true,
        Some(_) => return Err(Box::new(usage_error(program))),
        None => false,
    };
    reject_extra_args(program, args)?;
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
    Ok(())
}

fn run_rollup_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rollup_path = one_arg(program, args)?;
    let check = validate_remote_free_service_telemetry_collection_summary_rollup_artifact(
        Path::new(&rollup_path),
    )?;
    println!("{check}");
    println!(
        "{}",
        format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?
    );
    Ok(())
}

fn run_rollup_check_json_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = one_arg(program, args)?;
    let log_text = fs::read_to_string(&log_path)?;
    let check = parse_rollup_check_json_text(&log_text)?;
    println!("{check}");
    Ok(())
}

fn run_rollup_check_json_summary_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = one_arg(program, args)?;
    let log_text = fs::read_to_string(&log_path)?;
    let summary = summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log(
        &log_text,
    )?;
    println!("{summary}");
    println!(
        "{}",
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line(
            &summary,
        )?
    );
    Ok(())
}

fn run_rollup_check_json_summary_verify_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = one_arg(program, args)?;
    let log_text = fs::read_to_string(&log_path)?;
    let summary = parse_rollup_check_log_summary_json_text(&log_text)?;
    println!("{summary}");
    Ok(())
}

fn run_rollup_check_json_summary_verify_against_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (source_log_path, summary_log_path) = two_args(program, args)?;
    let summary =
        verify_rollup_check_log_summary_json_against_paths(&source_log_path, &summary_log_path)?;
    println!("{summary}");
    Ok(())
}

fn run_rollup_check_json_summary_verify_against_json_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (source_log_path, summary_log_path) = two_args(program, args)?;
    let report =
        rollup_check_log_summary_json_verification_report(&source_log_path, &summary_log_path)?;
    println!("{report}");
    println!(
        "{}",
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line(
            &report,
        )?
    );
    Ok(())
}

fn run_rollup_check_json_summary_verdict_rollup_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = one_arg(program, args)?;
    let log_text = fs::read_to_string(&log_path)?;
    let rollup =
        summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log(
            &log_text,
        )?;
    println!("{rollup}");
    println!(
        "{}",
        format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line(
            &rollup,
        )?
    );
    Ok(())
}

fn run_rollup_check_json_summary_verdict_rollup_verify_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = one_arg(program, args)?;
    let log_text = fs::read_to_string(&log_path)?;
    let rollup =
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
            &log_text,
        )?;
    println!("{rollup}");
    Ok(())
}

fn run_rollup_check_json_summary_verdict_rollup_verify_against_mode(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (source_log_path, rollup_log_path) = two_args(program, args)?;
    let source_log_text = fs::read_to_string(&source_log_path)?;
    let rollup_log_text = fs::read_to_string(&rollup_log_path)?;
    let rollup =
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log(
            &source_log_text,
            &rollup_log_text,
        )?;
    println!("{rollup}");
    Ok(())
}

fn one_arg(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let arg = args.next().ok_or_else(|| usage_error(program))?;
    reject_extra_args(program, args)?;
    Ok(arg)
}

fn two_args(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let first = args.next().ok_or_else(|| usage_error(program))?;
    let second = args.next().ok_or_else(|| usage_error(program))?;
    reject_extra_args(program, args)?;
    Ok((first, second))
}

fn reject_extra_args(
    program: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        return Err(Box::new(usage_error(program)));
    }
    Ok(())
}

fn validate_and_print_summary_path(summary_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let summary_path = PathBuf::from(summary_path);
    let report = validate_summary_path(&summary_path)?;

    println!("{}", collection_summary_validation_line(&report));
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
    host: Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
    output_count: usize,
    artifact_report: String,
    validation_summary_report: String,
    stability_output: String,
    timing_ranges: usize,
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
        host: summary.host,
        output_count: summary.output_count,
        artifact_report: artifact_report.to_string(),
        validation_summary_report,
        timing_ranges: stability_output.timing_ranges,
        stability_output: stability_output.text,
    })
}

fn collection_summary_validation_line(report: &BundleValidationReport) -> String {
    format!(
        "remote_free_service_telemetry_collection_summary_validation=ok summary={} manifest={} collection_mode={} run_id={} {} output_count={}",
        report.summary_path.display(),
        report.manifest_path.display(),
        report.collection_mode,
        report.run_id,
        summary_host_fields(report.host.as_ref()),
        report.output_count
    )
}

fn summary_host_fields(host: Option<&RemoteFreeServiceTelemetryCollectionSummaryHost>) -> String {
    match host {
        Some(host) => format!(
            "host_present=true host_os={} host_arch={} host_hostname={}",
            output_token(&host.os),
            output_token(&host.arch),
            host.hostname
                .as_deref()
                .map_or_else(|| "none".to_owned(), output_token)
        ),
        None => "host_present=false".to_owned(),
    }
}

fn output_token(value: &str) -> String {
    let token = value
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || matches!(value, '.' | '_' | '-') {
                value
            } else {
                '_'
            }
        })
        .collect::<String>();
    if token.is_empty() {
        "none".to_owned()
    } else {
        token
    }
}

fn parse_rollup_check_json_text(
    input: &str,
) -> Result<locus_validate::RemoteFreeServiceTelemetryCollectionSummaryRollupCheck, Box<dyn Error>>
{
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('{') {
            return Ok(
                parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line(
                    line,
                )?,
            );
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "missing remote-free service telemetry rollup check JSON line",
    )))
}

fn parse_rollup_check_log_summary_json_text(
    input: &str,
) -> Result<
    locus_validate::RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    Box<dyn Error>,
> {
    Ok(
        parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
            input,
        )?,
    )
}

fn verify_rollup_check_log_summary_json_against_paths(
    source_log_path: &str,
    summary_log_path: &str,
) -> Result<
    locus_validate::RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary,
    Box<dyn Error>,
> {
    let source_log_text = fs::read_to_string(source_log_path)?;
    let summary_log_text = fs::read_to_string(summary_log_path)?;
    Ok(
        verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
            &source_log_text,
            &summary_log_text,
        )?,
    )
}

fn rollup_check_log_summary_json_verification_report(
    source_log_path: &str,
    summary_log_path: &str,
) -> Result<
    locus_validate::RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification,
    Box<dyn Error>,
> {
    let source_log_text = fs::read_to_string(source_log_path)?;
    let summary_log_text = fs::read_to_string(summary_log_path)?;
    Ok(
        check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log(
            &source_log_text,
            &summary_log_text,
        )?,
    )
}

fn validate_summary_directory(
    root: &Path,
) -> Result<RemoteFreeServiceTelemetryCollectionSummaryRollup, Box<dyn Error>> {
    let mut rollup = build_remote_free_service_telemetry_collection_summary_directory_rollup(
        root,
        |summary_path| match validate_summary_path(summary_path) {
            Ok(report) => RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                run_id: Some(report.run_id),
                host: report.host,
                status: RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus::Valid,
                timing_ranges: report.timing_ranges,
            },
            Err(error) => {
                let message = error.to_string();
                let (run_id, host) = read_summary_identity(summary_path)
                    .map_or((None, None), |identity| {
                        (Some(identity.run_id), identity.host)
                    });
                RemoteFreeServiceTelemetryCollectionSummaryBundleValidation {
                    run_id,
                    host,
                    status: classify_validation_error(&message),
                    timing_ranges: 0,
                }
            }
        },
    )?;
    rollup.host = Some(current_rollup_host_metadata());
    Ok(rollup)
}

fn current_rollup_host_metadata() -> RemoteFreeServiceTelemetryCollectionSummaryRollupHost {
    RemoteFreeServiceTelemetryCollectionSummaryRollupHost {
        os: env::consts::OS.to_owned(),
        arch: env::consts::ARCH.to_owned(),
        hostname: env::var("HOSTNAME")
            .or_else(|_| env::var("COMPUTERNAME"))
            .ok()
            .filter(|hostname| !hostname.is_empty()),
    }
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

struct SummaryIdentity {
    run_id: String,
    host: Option<RemoteFreeServiceTelemetryCollectionSummaryHost>,
}

fn read_summary_identity(summary_path: &Path) -> Result<SummaryIdentity, Box<dyn Error>> {
    let summary_text = fs::read_to_string(summary_path)?;
    let summary = parse_remote_free_service_telemetry_collection_summary(&summary_text)?;
    Ok(SummaryIdentity {
        run_id: summary.run_id,
        host: summary.host,
    })
}

fn write_directory_rollup_artifact(
    root: &Path,
    rollup: &RemoteFreeServiceTelemetryCollectionSummaryRollup,
) -> Result<PathBuf, Box<dyn Error>> {
    Ok(write_remote_free_service_telemetry_collection_summary_rollup_artifact(root, rollup)?)
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
            "usage: {program} <collection-summary.json>\n       {program} --dir <evidence-root> [--write-rollup]\n       {program} --rollup <collection-summary-rollup.json>\n       {program} --rollup-check-json <saved-log.txt>\n       {program} --rollup-check-json-summary <saved-log.txt>\n       {program} --rollup-check-json-summary-verify <saved-log.txt>\n       {program} --rollup-check-json-summary-verify-against <saved-rollup-check-log.txt> <saved-summary-log.txt>\n       {program} --rollup-check-json-summary-verify-against-json <saved-rollup-check-log.txt> <saved-summary-log.txt>\n       {program} --rollup-check-json-summary-verdict-rollup <saved-verdict-log.txt>\n       {program} --rollup-check-json-summary-verdict-rollup-verify <saved-verdict-rollup-log.txt>\n       {program} --rollup-check-json-summary-verdict-rollup-verify-against <saved-verdict-log.txt> <saved-verdict-rollup-log.txt>"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        collection_summary_validation_line, compare_validation_summary,
        parse_rollup_check_json_text, summary_host_fields, validate_summary_directory,
        write_directory_rollup_artifact, BundleValidationReport,
    };
    use locus_validate::{
        format_remote_free_service_telemetry_collection_summary_rollup_check_json_line,
        validate_remote_free_service_telemetry_collection_summary_rollup_artifact,
        RemoteFreeServiceTelemetryCollectionSummaryHost,
        REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA,
    };
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
            "host": {
                "os": "linux",
                "arch": "x86_64",
                "hostname": "bench-host-01"
            },
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
    fn formats_summary_validation_line_without_host_metadata() {
        let report = BundleValidationReport {
            summary_path: PathBuf::from("collection-summary.json"),
            manifest_path: PathBuf::from("manifest.txt"),
            collection_mode: "benchmark_capture".to_owned(),
            run_id: "run-1".to_owned(),
            host: None,
            output_count: 2,
            artifact_report: String::new(),
            validation_summary_report: String::new(),
            stability_output: String::new(),
            timing_ranges: 1,
        };

        assert_eq!(
            collection_summary_validation_line(&report),
            "remote_free_service_telemetry_collection_summary_validation=ok summary=collection-summary.json manifest=manifest.txt collection_mode=benchmark_capture run_id=run-1 host_present=false output_count=2"
        );
    }

    #[test]
    fn formats_summary_validation_line_with_host_metadata() {
        let report = BundleValidationReport {
            summary_path: PathBuf::from("collection-summary.json"),
            manifest_path: PathBuf::from("manifest.txt"),
            collection_mode: "benchmark_capture".to_owned(),
            run_id: "run-1".to_owned(),
            host: Some(RemoteFreeServiceTelemetryCollectionSummaryHost {
                os: "macos".to_owned(),
                arch: "aarch64".to_owned(),
                hostname: None,
            }),
            output_count: 2,
            artifact_report: String::new(),
            validation_summary_report: String::new(),
            stability_output: String::new(),
            timing_ranges: 1,
        };

        assert_eq!(
            collection_summary_validation_line(&report),
            "remote_free_service_telemetry_collection_summary_validation=ok summary=collection-summary.json manifest=manifest.txt collection_mode=benchmark_capture run_id=run-1 host_present=true host_os=macos host_arch=aarch64 host_hostname=none output_count=2"
        );
    }

    #[test]
    fn formats_host_fields_as_single_line_tokens() {
        let host = RemoteFreeServiceTelemetryCollectionSummaryHost {
            os: "linux".to_owned(),
            arch: "x86_64".to_owned(),
            hostname: Some("bench host 01".to_owned()),
        };

        assert_eq!(
            summary_host_fields(Some(&host)),
            "host_present=true host_os=linux host_arch=x86_64 host_hostname=bench_host_01"
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
        let host = rollup.host.as_ref().expect("host metadata");
        assert_eq!(host.os, std::env::consts::OS);
        assert_eq!(host.arch, std::env::consts::ARCH);
        assert!(rollup
            .bundles
            .iter()
            .all(|bundle| bundle.host.as_ref().is_some_and(|host| {
                host.os == "linux"
                    && host.arch == "x86_64"
                    && host.hostname.as_deref() == Some("bench-host-01")
            })));
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
        assert_eq!(artifact["host"]["os"], std::env::consts::OS);
        assert_eq!(artifact["host"]["arch"], std::env::consts::ARCH);
        assert_eq!(artifact["bundles"][0]["host"]["os"], "linux");
        assert_eq!(artifact["bundles"][0]["host"]["arch"], "x86_64");
        assert_eq!(artifact["bundles"][0]["host"]["hostname"], "bench-host-01");
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
        assert_eq!(
            check.schema,
            "locus.remote_free_service.telemetry.collection_summary_rollup.v2"
        );
        assert_eq!(check.artifact_bytes, fs::metadata(&path)?.len());
        assert!(check.artifact_fingerprint.starts_with("fnv1a64:"));
        assert_eq!(check.artifact_fingerprint.len(), 24);
        assert_eq!(check.summaries, 1);
        assert_eq!(check.valid_bundles, 1);
        assert_eq!(check.timing_ranges, 1);
        assert_eq!(check.bundles, 1);
        assert!(check.rollup_host_present);
        assert_eq!(check.bundle_hosts, 1);
        assert_eq!(check.bundle_hosts_missing, 0);
        let json_line_text =
            format_remote_free_service_telemetry_collection_summary_rollup_check_json_line(&check)?;
        assert!(!json_line_text.contains('\n'));
        let parsed_log = parse_rollup_check_json_text(&format!("{check}\n{json_line_text}\n"))?;
        assert_eq!(parsed_log, check);
        let json_line = serde_json::from_str::<serde_json::Value>(&json_line_text)?;
        assert_eq!(
            json_line["schema"],
            REMOTE_FREE_SERVICE_TELEMETRY_COLLECTION_SUMMARY_ROLLUP_CHECK_SCHEMA
        );
        assert_eq!(json_line["path"], path.display().to_string());
        assert_eq!(
            json_line["artifact_fingerprint"],
            check.artifact_fingerprint
        );
        assert_eq!(json_line["artifact"]["path"], path.display().to_string());
        assert_eq!(json_line["artifact"]["bytes"], check.artifact_bytes);
        assert_eq!(
            json_line["artifact"]["fingerprint"],
            check.artifact_fingerprint
        );
        assert_eq!(
            json_line["host_coverage"]["rollup_host_present"],
            check.rollup_host_present
        );
        assert_eq!(
            json_line["host_coverage"]["bundle_hosts"],
            check.bundle_hosts
        );
        assert_eq!(
            json_line["status_coverage"]["valid_bundles"],
            check.valid_bundles
        );

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
