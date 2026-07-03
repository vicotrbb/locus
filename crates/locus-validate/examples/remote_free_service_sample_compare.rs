#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs};

    use locus_validate::compare_remote_free_service_telemetry_sample_outputs;

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "remote_free_service_sample_compare".to_owned());
    let baseline_path = args.next().ok_or_else(|| usage_error(&program))?;
    let candidate_path = args.next().ok_or_else(|| usage_error(&program))?;
    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    let baseline_output = fs::read_to_string(baseline_path)?;
    let candidate_output = fs::read_to_string(candidate_path)?;
    let report =
        compare_remote_free_service_telemetry_sample_outputs(&baseline_output, &candidate_output)?;

    println!("{report}");
    for drift in report.drifts {
        println!("{drift}");
    }

    Ok(())
}

fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("usage: {program} <baseline-output> <candidate-output>"),
    )
}
