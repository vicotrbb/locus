#![allow(missing_docs)]

#[cfg(target_os = "linux")]
use locus::MappedScratchHugePageAdvice;

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus::sys::page_size;
    use locus::MappedScratchArena;
    use locus::NodeId;

    let mode = parse_mode_arg()?;
    let advice = mode.advice();
    let mut arena = MappedScratchArena::new(NodeId(0), 4 * 1024 * 1024)?;
    let mapping_start = arena.mapping_start_address();
    let base_page_size = page_size()?;
    let base_page_kb = base_page_size / 1024;

    println!("mapped_scratch_thp=started mode={}", mode.as_str());
    println!("mapping_start=0x{mapping_start:x}");
    println!("mapping_len={}", arena.mapping_len());
    println!("base_page_kb={base_page_kb}");

    match arena.advise_transparent_huge_pages(advice) {
        Ok(()) => println!("thp_advice=ok mode={}", mode.as_str()),
        Err(error) => {
            println!("thp_advice=error mode={}", mode.as_str());
            println!("thp_advice_error={error}");
            return Ok(());
        }
    }

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");

    print_thp_evidence(mapping_start, base_page_kb)?;

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
        .unwrap_or_else(|| "mapped_scratch_thp".to_owned());
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
fn print_numa_maps_evidence(
    address_match: locus_observe::NumaMapsAddressMatch<'_>,
    base_page_kb: usize,
) -> bool {
    let entry = address_match.entry;
    println!("numa_maps_match={}", address_match.kind);
    println!("numa_maps_policy={}", entry.policy);
    for (node, pages) in &entry.node_pages {
        println!("numa_maps_node={} pages={pages}", node.0);
    }

    match entry
        .attributes
        .get("kernelpagesize_kB")
        .and_then(|value| value.parse::<usize>().ok())
    {
        Some(page_kb) => {
            println!("kernel_page_kb={page_kb}");
            print_kernel_page_observation(page_kb, base_page_kb);
            true
        }
        None => {
            println!("numa_maps_kernel_page_kb=missing");
            false
        }
    }
}

#[cfg(target_os = "linux")]
fn print_thp_evidence(
    mapping_start: usize,
    base_page_kb: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use locus_observe::{
        numa_maps_entry_for_address, read_self_numa_maps, read_self_smaps, smaps_entry_for_address,
        ObserveReadError,
    };

    let mut unknown_reason = "numa_maps_unavailable";

    match read_self_numa_maps() {
        Ok(entries) => {
            println!("numa_maps=available entries={}", entries.len());
            if let Some(address_match) = numa_maps_entry_for_address(&entries, mapping_start) {
                if print_numa_maps_evidence(address_match, base_page_kb) {
                    return Ok(());
                }
                unknown_reason = "kernel_page_size_missing";
            } else {
                println!("numa_maps_match=missing");
                unknown_reason = "mapping_missing";
            }
        }
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
        }
        Err(error) => return Err(Box::new(error)),
    }

    match read_self_smaps() {
        Ok(entries) => {
            println!("smaps=available entries={}", entries.len());
            if let Some(entry) = smaps_entry_for_address(&entries, mapping_start) {
                if print_smaps_evidence(entry, base_page_kb) {
                    return Ok(());
                }
                unknown_reason = "kernel_page_size_missing";
            } else {
                println!("smaps_match=missing");
                unknown_reason = "mapping_missing";
            }
        }
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("smaps=unavailable");
            if unknown_reason == "numa_maps_unavailable" {
                unknown_reason = "observability_unavailable";
            }
        }
        Err(error) => return Err(Box::new(error)),
    }

    println!("kernel_page_kb=unknown");
    println!("thp_observed=unknown reason={unknown_reason}");
    Ok(())
}

#[cfg(target_os = "linux")]
fn print_smaps_evidence(entry: &locus_observe::SmapsEntry, base_page_kb: usize) -> bool {
    println!("smaps_match=containing_range");
    println!(
        "smaps_range=0x{:x}-0x{:x}",
        entry.start_address, entry.end_address
    );

    if let Some(page_kb) = entry
        .kernel_page_kb
        .and_then(|value| usize::try_from(value).ok())
    {
        println!("kernel_page_kb={page_kb}");
        print_kernel_page_observation(page_kb, base_page_kb);
        true
    } else {
        println!("smaps_kernel_page_kb=missing");
        false
    }
}

#[cfg(target_os = "linux")]
fn print_kernel_page_observation(page_kb: usize, base_page_kb: usize) {
    if page_kb > base_page_kb {
        println!("thp_observed=yes reason=kernel_page_size");
    } else {
        println!("thp_observed=no reason=base_page_size");
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mapped_scratch_thp=unsupported-platform");
}
