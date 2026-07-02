#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;

    use locus_validate::linux::{
        evaluate_placement_validation_outputs, PlacementValidationOutputs,
    };

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "placement_validation_gate".to_owned());
    let memory_policy_path = args.next().ok_or_else(|| usage_error(&program))?;
    let placement_readiness_path = args.next().ok_or_else(|| usage_error(&program))?;
    let placement_proof_path = args.next().ok_or_else(|| usage_error(&program))?;
    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    let memory_policy_output = fs::read_to_string(memory_policy_path)?;
    let placement_readiness_output = fs::read_to_string(placement_readiness_path)?;
    let placement_proof_output = fs::read_to_string(placement_proof_path)?;

    let gate = evaluate_placement_validation_outputs(PlacementValidationOutputs {
        memory_policy_output: &memory_policy_output,
        placement_readiness_output: &placement_readiness_output,
        placement_proof_output: &placement_proof_output,
    })?;

    println!(
        "placement_validation_gate={} reason={}",
        gate.status, gate.reason
    );

    Ok(())
}

#[cfg(target_os = "linux")]
fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "usage: {program} <memory-policy-output> <placement-readiness-output> <placement-proof-output>"
        ),
    )
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("placement_validation_gate=unsupported-platform");
}
