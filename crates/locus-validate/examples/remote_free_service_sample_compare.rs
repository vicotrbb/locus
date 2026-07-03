#![allow(missing_docs)]

use std::path::{Path, PathBuf};

use locus_validate::{
    compare_remote_free_service_telemetry_sample_outputs_with_timings,
    parse_remote_free_service_telemetry_timing_stability_manifest,
    summarize_remote_free_service_telemetry_timing_stability,
    RemoteFreeServiceTelemetryTimingStabilityRun,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs};

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_sample_compare".to_owned());
    let baseline_path = args.next().ok_or_else(|| usage_error(&program))?;
    if baseline_path == "--manifest" {
        let manifest_path = args.next().ok_or_else(|| usage_error(&program))?;
        if args.next().is_some() {
            return Err(Box::new(usage_error(&program)));
        }
        let manifest_text = fs::read_to_string(&manifest_path)?;
        let manifest =
            parse_remote_free_service_telemetry_timing_stability_manifest(&manifest_text)?;
        let manifest_path = PathBuf::from(manifest_path);
        let baseline_output = fs::read_to_string(resolve_manifest_path(
            &manifest_path,
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
            .map(|entry| fs::read_to_string(resolve_manifest_path(&manifest_path, &entry.path)))
            .collect::<Result<Vec<_>, _>>()?;

        print_stability_report(
            &manifest.baseline.label,
            &baseline_output,
            &candidate_labels,
            &candidate_outputs,
        )?;
        return Ok(());
    }

    let candidate_paths = args.collect::<Vec<_>>();
    if candidate_paths.is_empty() {
        return Err(Box::new(usage_error(&program)));
    }

    let baseline_output = fs::read_to_string(&baseline_path)?;
    let candidate_outputs = candidate_paths
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<_>, _>>()?;

    if candidate_paths.len() == 1 {
        let report = compare_remote_free_service_telemetry_sample_outputs_with_timings(
            &baseline_output,
            &candidate_outputs[0],
        )?;

        println!("{report}");
        for drift in report.samples.drifts {
            println!("{drift}");
        }
        for delta in report.timing_deltas {
            println!("{delta}");
        }

        return Ok(());
    }

    print_stability_report(
        &baseline_path,
        &baseline_output,
        &candidate_paths,
        &candidate_outputs,
    )?;

    Ok(())
}

fn print_stability_report(
    baseline_label: &str,
    baseline_output: &str,
    candidate_labels: &[String],
    candidate_outputs: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
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
            label: baseline_label,
            output: baseline_output,
        },
        &candidate_runs,
    )?;

    println!("{report}");
    for discard in report.discards {
        println!("{discard}");
    }
    for range in report.ranges {
        println!("{range}");
    }

    Ok(())
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

fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "usage: {program} <baseline-output> <candidate-output> [candidate-output ...]\n       {program} --manifest <manifest>"
        ),
    )
}
