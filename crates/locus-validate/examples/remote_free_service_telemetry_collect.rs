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
    remote_free_service_telemetry_repeated_capture_labels,
    summarize_remote_free_service_telemetry_timing_stability,
    RemoteFreeServiceTelemetryCollectionSummaryHost, RemoteFreeServiceTelemetryTimingStabilityRun,
};
use serde_json::{json, Value};

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
    let repeat_count = if collection_mode == CollectionMode::BenchmarkCapture
        && args.first().is_some_and(|arg| arg == "--repeat")
    {
        if args.len() < 2 {
            return Err(Box::new(usage_error(&program)));
        }
        let repeat_count = args[1].parse::<usize>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid repeated benchmark capture count: {error}"),
            )
        })?;
        args.drain(0..2);
        Some(repeat_count)
    } else {
        None
    };
    let criterion_args = split_criterion_args(&mut args);

    let (evidence_root, baseline, candidates) = match repeat_count {
        Some(count) => repeated_benchmark_sources(&program, &args, count)?,
        None => explicit_sources(&program, &args)?,
    };
    let candidate_labels = candidates
        .iter()
        .map(|candidate| candidate.label.clone())
        .collect::<Vec<_>>();

    let manifest = format_remote_free_service_telemetry_timing_stability_manifest(
        &baseline.label,
        &candidate_labels,
    )?;

    fs::create_dir_all(&evidence_root)?;
    let run_dir = evidence_root.join(&run_id);
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
    let collection_summary_path = write_collection_summary(
        &run_dir,
        &run_id,
        collection_mode,
        &baseline,
        &candidates,
        &criterion_args,
    )?;

    println!(
        "remote_free_service_telemetry_evidence_collection mode={} directory={} manifest={} validation_summary={} collection_summary={} outputs={}",
        collection_mode.as_str(),
        run_dir.display(),
        manifest_path.display(),
        summary_path.display(),
        collection_summary_path.display(),
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

fn explicit_sources(
    program: &str,
    args: &[String],
) -> Result<(PathBuf, SourceRun, Vec<SourceRun>), Box<dyn std::error::Error>> {
    if args.len() < 5 || (args.len() - 3) % 2 != 0 {
        return Err(Box::new(usage_error(program)));
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

    Ok((evidence_root, baseline, candidates))
}

fn repeated_benchmark_sources(
    program: &str,
    args: &[String],
    count: usize,
) -> Result<(PathBuf, SourceRun, Vec<SourceRun>), Box<dyn std::error::Error>> {
    if args.len() != 3 {
        return Err(Box::new(usage_error(program)));
    }

    let evidence_root = PathBuf::from(&args[0]);
    let labels = remote_free_service_telemetry_repeated_capture_labels(&args[1], count)?;
    let benchmark_filter = &args[2];
    let baseline = SourceRun {
        label: labels[0].clone(),
        input: benchmark_filter.clone(),
    };
    let candidates = labels[1..]
        .iter()
        .map(|label| SourceRun {
            label: label.clone(),
            input: benchmark_filter.clone(),
        })
        .collect::<Vec<_>>();

    Ok((evidence_root, baseline, candidates))
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

fn write_collection_summary(
    run_dir: &Path,
    run_id: &str,
    mode: CollectionMode,
    baseline: &SourceRun,
    candidates: &[SourceRun],
    criterion_args: &[String],
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut sources = Vec::with_capacity(candidates.len() + 1);
    sources.push(source_json("baseline", baseline));
    sources.extend(
        candidates
            .iter()
            .map(|candidate| source_json("candidate", candidate)),
    );

    let mut artifacts = Vec::with_capacity(candidates.len() + 3);
    artifacts.push(output_artifact_json(run_dir, "baseline", baseline)?);
    for candidate in candidates {
        artifacts.push(output_artifact_json(run_dir, "candidate", candidate)?);
    }
    artifacts.push(artifact_json(run_dir, "manifest", None, "manifest.txt")?);
    artifacts.push(artifact_json(
        run_dir,
        "validation_summary",
        None,
        "validation-summary.txt",
    )?);
    let host = current_collection_summary_host_metadata();

    let summary = json!({
        "schema": "locus.remote_free_service.telemetry.collection_summary.v1",
        "collection_mode": mode.as_str(),
        "run_id": run_id,
        "host": collection_summary_host_json(&host),
        "output_count": candidates.len() + 1,
        "criterion_args": criterion_args,
        "sources": sources,
        "artifacts": artifacts,
    });
    let output = format!("{}\n", serde_json::to_string_pretty(&summary)?);
    let path = run_dir.join("collection-summary.json");
    fs::write(&path, output)?;
    Ok(path)
}

fn current_collection_summary_host_metadata() -> RemoteFreeServiceTelemetryCollectionSummaryHost {
    RemoteFreeServiceTelemetryCollectionSummaryHost {
        os: env::consts::OS.to_owned(),
        arch: env::consts::ARCH.to_owned(),
        hostname: env::var("HOSTNAME")
            .or_else(|_| env::var("COMPUTERNAME"))
            .ok()
            .filter(|hostname| !hostname.is_empty()),
    }
}

fn collection_summary_host_json(host: &RemoteFreeServiceTelemetryCollectionSummaryHost) -> Value {
    json!({
        "os": host.os.as_str(),
        "arch": host.arch.as_str(),
        "hostname": host.hostname.as_deref(),
    })
}

fn source_json(role: &str, source: &SourceRun) -> Value {
    json!({
        "role": role,
        "label": source.label,
        "input": source.input,
        "artifact": format!("{}.txt", source.label),
    })
}

fn output_artifact_json(
    run_dir: &Path,
    role: &str,
    source: &SourceRun,
) -> Result<Value, Box<dyn std::error::Error>> {
    artifact_json(
        run_dir,
        "output",
        Some(role),
        &format!("{}.txt", source.label),
    )
}

fn artifact_json(
    run_dir: &Path,
    kind: &str,
    role: Option<&str>,
    relative_path: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let byte_count = fs::metadata(run_dir.join(relative_path))?.len();
    let mut artifact = json!({
        "kind": kind,
        "path": relative_path,
        "byte_count": byte_count,
    });
    if let Some(role) = role {
        artifact["role"] = json!(role);
    }
    Ok(artifact)
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
            "usage: {program} [--run-id <run-id>] <evidence-root> <baseline-label> <baseline-output> <candidate-label> <candidate-output> [candidate-label candidate-output ...]\n       {program} [--run-id <run-id>] --bench <evidence-root> <baseline-label> <baseline-filter> <candidate-label> <candidate-filter> [candidate-label candidate-filter ...] [-- <criterion-arg> ...]\n       {program} [--run-id <run-id>] --bench --repeat <count> <evidence-root> <base-label> <benchmark-filter> [-- <criterion-arg> ...]"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{write_collection_summary, CollectionMode, SourceRun};
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn writes_collection_summary_json_with_artifact_byte_counts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let run_dir = env::temp_dir().join(format!(
            "locus-collection-summary-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&run_dir)?;

        fs::write(run_dir.join("run-01.txt"), "aaa")?;
        fs::write(run_dir.join("run-02.txt"), "bbbb")?;
        fs::write(run_dir.join("manifest.txt"), "manifest")?;
        fs::write(run_dir.join("validation-summary.txt"), "summary")?;

        let baseline = SourceRun {
            label: "run-01".to_owned(),
            input: "benchmark_filter".to_owned(),
        };
        let candidates = [SourceRun {
            label: "run-02".to_owned(),
            input: "benchmark_filter".to_owned(),
        }];
        let criterion_args = ["--sample-size".to_owned(), "10".to_owned()];
        let summary_path = write_collection_summary(
            &run_dir,
            "run-id",
            CollectionMode::BenchmarkCapture,
            &baseline,
            &candidates,
            &criterion_args,
        )?;

        let summary = fs::read_to_string(&summary_path)?;
        let summary: serde_json::Value = serde_json::from_str(&summary)?;
        assert_eq!(
            summary["schema"],
            "locus.remote_free_service.telemetry.collection_summary.v1"
        );
        assert_eq!(summary["collection_mode"], "benchmark_capture");
        assert_eq!(summary["run_id"], "run-id");
        assert_eq!(summary["host"]["os"], std::env::consts::OS);
        assert_eq!(summary["host"]["arch"], std::env::consts::ARCH);
        assert_eq!(summary["output_count"], 2);
        assert_eq!(
            summary["criterion_args"],
            serde_json::json!(["--sample-size", "10"])
        );
        assert_eq!(summary["sources"][0]["artifact"], "run-01.txt");
        assert_eq!(summary["sources"][1]["artifact"], "run-02.txt");
        assert_eq!(summary["artifacts"][0]["byte_count"], 3);
        assert_eq!(summary["artifacts"][1]["byte_count"], 4);
        assert_eq!(summary["artifacts"][2]["byte_count"], 8);
        assert_eq!(summary["artifacts"][3]["byte_count"], 7);

        fs::remove_dir_all(run_dir)?;
        Ok(())
    }
}
