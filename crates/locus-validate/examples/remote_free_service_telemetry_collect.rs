#![allow(missing_docs)]

use std::{
    env,
    fmt::Write as _,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use locus_validate::{
    format_remote_free_service_telemetry_timing_stability_manifest,
    summarize_remote_free_service_telemetry_timing_stability,
    RemoteFreeServiceTelemetryTimingStabilityRun,
};

#[derive(Debug)]
struct SourceRun {
    label: String,
    input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectionMode {
    SavedOutput,
    BenchmarkCapture,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_telemetry_collect".to_owned());
    let mut args = args.collect::<Vec<_>>();

    let run_id = if args.first().is_some_and(|arg| arg == "--run-id") {
        if args.len() < 2 {
            return Err(Box::new(usage_error(&program)));
        }
        let run_id = args[1].clone();
        args.drain(0..2);
        run_id
    } else {
        format!(
            "remote-free-service-telemetry-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        )
    };
    validate_artifact_label(&run_id)?;

    let collection_mode = if args.first().is_some_and(|arg| arg == "--bench") {
        args.remove(0);
        CollectionMode::BenchmarkCapture
    } else {
        CollectionMode::SavedOutput
    };
    let criterion_args = split_criterion_args(&mut args);

    if args.len() < 5 || (args.len() - 3) % 2 != 0 {
        return Err(Box::new(usage_error(&program)));
    }

    let evidence_root = PathBuf::from(&args[0]);
    let baseline = SourceRun {
        label: args[1].clone(),
        input: args[2].clone(),
    };
    let candidates = args[3..]
        .chunks_exact(2)
        .map(|chunk| SourceRun {
            label: chunk[0].clone(),
            input: chunk[1].clone(),
        })
        .collect::<Vec<_>>();
    let candidate_labels = candidates
        .iter()
        .map(|candidate| candidate.label.clone())
        .collect::<Vec<_>>();

    let manifest = format_remote_free_service_telemetry_timing_stability_manifest(
        &baseline.label,
        &candidate_labels,
    )?;

    fs::create_dir_all(&evidence_root)?;
    let run_dir = evidence_root.join(run_id);
    fs::create_dir(&run_dir)?;

    let baseline_output =
        collect_labeled_output(collection_mode, &run_dir, &baseline, &criterion_args)?;
    let candidate_outputs = candidates
        .iter()
        .map(|candidate| {
            collect_labeled_output(collection_mode, &run_dir, candidate, &criterion_args)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let manifest_path = run_dir.join("manifest.txt");
    fs::write(&manifest_path, manifest)?;

    let summary = stability_summary_text(
        &baseline.label,
        &baseline_output,
        &candidate_labels,
        &candidate_outputs,
    )?;
    let summary_path = run_dir.join("validation-summary.txt");
    fs::write(&summary_path, &summary)?;

    println!(
        "remote_free_service_telemetry_evidence_collection mode={} directory={} manifest={} validation_summary={} outputs={}",
        collection_mode.as_str(),
        run_dir.display(),
        manifest_path.display(),
        summary_path.display(),
        candidate_outputs.len() + 1
    );
    print!("{summary}");
    io::stdout().flush()?;

    Ok(())
}

impl CollectionMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::SavedOutput => "saved_output",
            Self::BenchmarkCapture => "benchmark_capture",
        }
    }
}

fn split_criterion_args(args: &mut Vec<String>) -> Vec<String> {
    let Some(separator_index) = args.iter().position(|arg| arg == "--") else {
        return Vec::new();
    };
    let criterion_args = args.split_off(separator_index + 1);
    let _ = args.pop();
    criterion_args
}

fn collect_labeled_output(
    mode: CollectionMode,
    run_dir: &Path,
    source: &SourceRun,
    criterion_args: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    match mode {
        CollectionMode::SavedOutput => copy_labeled_output(run_dir, source),
        CollectionMode::BenchmarkCapture => {
            capture_labeled_benchmark_output(run_dir, source, criterion_args)
        }
    }
}

fn copy_labeled_output(
    run_dir: &Path,
    source: &SourceRun,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_artifact_label(&source.label)?;
    let output = fs::read_to_string(&source.input)?;
    fs::write(run_dir.join(format!("{}.txt", source.label)), &output)?;
    Ok(output)
}

fn capture_labeled_benchmark_output(
    run_dir: &Path,
    source: &SourceRun,
    criterion_args: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    validate_artifact_label(&source.label)?;

    let mut command = Command::new("cargo");
    command
        .args([
            "bench",
            "-p",
            "locus-alloc",
            "--bench",
            "remote_free_service_telemetry",
            source.input.as_str(),
        ])
        .env("LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON", "1");
    if !criterion_args.is_empty() {
        command.arg("--").args(criterion_args);
    }

    let output = command.output()?;
    let mut captured = String::new();
    captured.push_str(&String::from_utf8_lossy(&output.stdout));
    captured.push_str(&String::from_utf8_lossy(&output.stderr));
    fs::write(run_dir.join(format!("{}.txt", source.label)), &captured)?;

    if !output.status.success() {
        return Err(Box::new(io::Error::other(format!(
            "remote-free service telemetry benchmark failed for label={} filter={} status={}",
            source.label, source.input, output.status
        ))));
    }

    Ok(captured)
}

fn stability_summary_text(
    baseline_label: &str,
    baseline_output: &str,
    candidate_labels: &[String],
    candidate_outputs: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
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

fn validate_artifact_label(label: &str) -> Result<(), io::Error> {
    if label.is_empty()
        || label == "."
        || label == ".."
        || !label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid artifact label: {label}"),
        ));
    }
    Ok(())
}

fn usage_error(program: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "usage: {program} [--run-id <run-id>] <evidence-root> <baseline-label> <baseline-output> <candidate-label> <candidate-output> [candidate-label candidate-output ...]\n       {program} [--run-id <run-id>] --bench <evidence-root> <baseline-label> <baseline-filter> <candidate-label> <candidate-filter> [candidate-label candidate-filter ...] [-- <criterion-arg> ...]"
        ),
    )
}
