#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs};

    use locus_validate::{
        compare_remote_free_service_telemetry_sample_outputs_with_timings,
        summarize_remote_free_service_telemetry_timing_stability,
        RemoteFreeServiceTelemetryTimingStabilityRun,
    };

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_sample_compare".to_owned());
    let baseline_path = args.next().ok_or_else(|| usage_error(&program))?;
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

    let candidate_runs = candidate_paths
        .iter()
        .zip(candidate_outputs.iter())
        .map(|(label, output)| RemoteFreeServiceTelemetryTimingStabilityRun { label, output })
        .collect::<Vec<_>>();
    let report = summarize_remote_free_service_telemetry_timing_stability(
        RemoteFreeServiceTelemetryTimingStabilityRun {
            label: &baseline_path,
            output: &baseline_output,
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

fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("usage: {program} <baseline-output> <candidate-output> [candidate-output ...]"),
    )
}
