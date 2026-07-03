#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;

    use locus_validate::summarize_mapped_scratch_thp_benchmark_evidence_report_lines;

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "mapped_scratch_thp_benchmark_evidence_run_summary".to_owned());
    let report_paths = args.collect::<Vec<_>>();
    if report_paths.is_empty() {
        return Err(Box::new(usage_error(&program)));
    }

    let mut reports = String::new();
    for path in report_paths {
        reports.push_str(&fs::read_to_string(path)?);
        reports.push('\n');
    }

    let summary = summarize_mapped_scratch_thp_benchmark_evidence_report_lines(&reports)?;
    println!("{summary}");

    Ok(())
}

fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("usage: {program} <compact-thp-report> [compact-thp-report ...]"),
    )
}
