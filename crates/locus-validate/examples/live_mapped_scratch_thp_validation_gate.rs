#![allow(missing_docs)]

#[cfg(target_os = "linux")]
use locus_alloc::MappedScratchHugePageAdvice;

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use locus_alloc::MappedScratchArena;
    use locus_core::NodeId;
    use locus_observe::{numa_maps_entry_for_address, read_self_numa_maps, ObserveReadError};
    use locus_sys::page_size;
    use locus_validate::evaluate_mapped_scratch_thp_validation_output;

    let mode = parse_mode_arg()?;
    let advice = mode.advice();
    let mut arena = MappedScratchArena::new(NodeId(0), 4 * 1024 * 1024)?;
    let mapping_start = arena.mapping_start_address();
    let base_page_size = page_size()?;
    let base_page_kb = base_page_size / 1024;
    let mut output = String::new();

    emit_line(
        &mut output,
        format_args!("mapped_scratch_thp=started mode={}", mode.as_str()),
    );
    emit_line(
        &mut output,
        format_args!("mapping_start=0x{mapping_start:x}"),
    );
    emit_line(
        &mut output,
        format_args!("mapping_len={}", arena.mapping_len()),
    );
    emit_line(&mut output, format_args!("base_page_kb={base_page_kb}"));

    match arena.advise_transparent_huge_pages(advice) {
        Ok(()) => emit_line(
            &mut output,
            format_args!("thp_advice=ok mode={}", mode.as_str()),
        ),
        Err(error) => {
            emit_line(
                &mut output,
                format_args!("thp_advice=error mode={}", mode.as_str()),
            );
            emit_line(&mut output, format_args!("thp_advice_error={error}"));
            let gate = evaluate_mapped_scratch_thp_validation_output(&output)?;
            println!("{gate}");
            return Ok(());
        }
    }

    let touched = arena.write_touch_pages()?;
    emit_line(&mut output, format_args!("touched={touched}"));

    match read_self_numa_maps() {
        Ok(entries) => {
            emit_line(
                &mut output,
                format_args!("numa_maps=available entries={}", entries.len()),
            );
            if let Some(address_match) = numa_maps_entry_for_address(&entries, mapping_start) {
                emit_numa_maps_evidence(&mut output, address_match, base_page_kb);
            } else {
                emit_line(&mut output, format_args!("numa_maps_match=missing"));
                emit_line(
                    &mut output,
                    format_args!("thp_observed=unknown reason=mapping_missing"),
                );
            }
        }
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            emit_line(&mut output, format_args!("numa_maps=unavailable"));
            emit_line(
                &mut output,
                format_args!("thp_observed=unknown reason=numa_maps_unavailable"),
            );
        }
        Err(error) => return Err(Box::new(error)),
    }

    let gate = evaluate_mapped_scratch_thp_validation_output(&output)?;
    println!("{gate}");

    Ok(())
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
enum ThpMode {
    HugePage,
    NoHugePage,
}

#[cfg(target_os = "linux")]
impl ThpMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::HugePage => "hugepage",
            Self::NoHugePage => "no_hugepage",
        }
    }

    fn advice(self) -> MappedScratchHugePageAdvice {
        match self {
            Self::HugePage => MappedScratchHugePageAdvice::HugePage,
            Self::NoHugePage => MappedScratchHugePageAdvice::NoHugePage,
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_mode_arg() -> Result<ThpMode, Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "live_mapped_scratch_thp_validation_gate".to_owned());
    let mode = match args.next().as_deref() {
        None | Some("hugepage") => ThpMode::HugePage,
        Some("no_hugepage") => ThpMode::NoHugePage,
        Some(_) => return Err(Box::new(usage_error(&program))),
    };
    if args.next().is_some() {
        return Err(Box::new(usage_error(&program)));
    }
    Ok(mode)
}

#[cfg(target_os = "linux")]
fn usage_error(program: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("usage: {program} [hugepage|no_hugepage]"),
    )
}

#[cfg(target_os = "linux")]
fn emit_numa_maps_evidence(
    output: &mut String,
    address_match: locus_observe::NumaMapsAddressMatch<'_>,
    base_page_kb: usize,
) {
    let entry = address_match.entry;
    emit_line(
        output,
        format_args!("numa_maps_match={}", address_match.kind),
    );
    emit_line(output, format_args!("numa_maps_policy={}", entry.policy));
    for (node, pages) in &entry.node_pages {
        emit_line(
            output,
            format_args!("numa_maps_node={} pages={pages}", node.0),
        );
    }

    match entry
        .attributes
        .get("kernelpagesize_kB")
        .and_then(|value| value.parse::<usize>().ok())
    {
        Some(page_kb) => {
            emit_line(output, format_args!("kernel_page_kb={page_kb}"));
            if page_kb > base_page_kb {
                emit_line(
                    output,
                    format_args!("thp_observed=yes reason=kernel_page_size"),
                );
            } else {
                emit_line(
                    output,
                    format_args!("thp_observed=no reason=base_page_size"),
                );
            }
        }
        None => {
            emit_line(output, format_args!("kernel_page_kb=unknown"));
            emit_line(
                output,
                format_args!("thp_observed=unknown reason=kernel_page_size_missing"),
            );
        }
    }
}

#[cfg(target_os = "linux")]
fn emit_line(output: &mut String, args: std::fmt::Arguments<'_>) {
    let line = args.to_string();
    println!("{line}");
    output.push_str(&line);
    output.push('\n');
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_validate::evaluate_mapped_scratch_thp_validation_output;

    let output = "mapped_scratch_thp=unsupported-platform\n";
    print!("{output}");
    let gate = evaluate_mapped_scratch_thp_validation_output(output)?;
    println!("{gate}");

    Ok(())
}
