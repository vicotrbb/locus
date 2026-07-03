#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;

    use locus_validate::evaluate_mapped_scratch_thp_validation_output;

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "mapped_scratch_thp_validation_gate".to_owned());
    let mapped_scratch_thp_path = args.next().ok_or_else(|| usage_error(&program))?;
    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }

    let mapped_scratch_thp_output = fs::read_to_string(mapped_scratch_thp_path)?;
    let gate = evaluate_mapped_scratch_thp_validation_output(&mapped_scratch_thp_output)?;
    println!("{gate}");

    Ok(())
}

fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("usage: {program} <mapped-scratch-thp-output>"),
    )
}
