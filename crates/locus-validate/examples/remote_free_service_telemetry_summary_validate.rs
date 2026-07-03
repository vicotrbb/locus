#![allow(missing_docs)]

use std::{
    env,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use locus_validate::{
    parse_remote_free_service_telemetry_collection_summary,
    parse_remote_free_service_telemetry_timing_stability_manifest,
    resolve_remote_free_service_telemetry_collection_summary_manifest_path,
    summarize_remote_free_service_telemetry_timing_stability,
    verify_remote_free_service_telemetry_collection_summary_artifacts,
    RemoteFreeServiceTelemetryTimingStabilityRun,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_telemetry_summary_validate".to_owned());
    let summary_path = args.next().ok_or_else(|| usage_error(&program))?;
    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    let summary_path = PathBuf::from(summary_path);
    let summary_text = fs::read_to_string(&summary_path)?;
    let summary = parse_remote_free_service_telemetry_collection_summary(&summary_text)?;
    let artifact_report =
        verify_remote_free_service_telemetry_collection_summary_artifacts(&summary_path, &summary)?;
    let manifest_path = resolve_remote_free_service_telemetry_collection_summary_manifest_path(
        &summary_path,
        &summary,
    )?;
    let stability_output = stability_report_from_manifest(&manifest_path)?;

    println!(
        "remote_free_service_telemetry_collection_summary_validation=ok summary={} manifest={} collection_mode={} run_id={} output_count={}",
        summary_path.display(),
        manifest_path.display(),
        summary.collection_mode,
        summary.run_id,
        summary.output_count
    );
    println!("{artifact_report}");
    print!("{stability_output}");

    Ok(())
}

fn stability_report_from_manifest(
    manifest_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
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
    for range in report.ranges {
        let _ = writeln!(&mut output, "{range}");
    }

    Ok(output)
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
        format!("usage: {program} <collection-summary.json>"),
    )
}
