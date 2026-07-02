#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use locus_alloc::MappedScratchArena;
    use locus_core::NodeId;
    use locus_observe::{
        numa_maps_entry_by_start_address, numa_maps_entry_containing_address, read_self_numa_maps,
        ObserveReadError,
    };

    let mut arena = MappedScratchArena::new(NodeId(0), 16 * 1024)?;
    let mapping_start = arena.mapping_start_address();

    println!("mapping_start=0x{mapping_start:x}");
    println!("mapping_len={}", arena.mapping_len());

    match arena.bind_to_node(NodeId(0)) {
        Ok(()) => println!("mapped_scratch_bind=ok"),
        Err(error) => println!("mapped_scratch_bind=error {error}"),
    }

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");
    println!("home_node={}", arena.home_node().0);

    match read_self_numa_maps() {
        Ok(entries) => {
            if let Some(entry) = numa_maps_entry_by_start_address(&entries, mapping_start) {
                print_placement("ok", entry, arena.home_node());
            } else if let Some(entry) = numa_maps_entry_containing_address(&entries, mapping_start)
            {
                print_placement("containing", entry, arena.home_node());
            } else {
                println!("numa_maps_match=missing");
            }
        }
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
        }
        Err(error) => return Err(Box::new(error)),
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_placement(
    match_status: &str,
    entry: &locus_observe::NumaMapsEntry,
    expected_node: locus_core::NodeId,
) {
    let evidence = locus_observe::NumaPlacementEvidence::from_entry(entry, expected_node);
    println!(
        "numa_maps_match={match_status} policy={} placement_status={} expected_node={} expected_pages={} total_pages={}",
        entry.policy,
        evidence.status,
        evidence.expected_node.0,
        evidence.expected_node_pages,
        evidence.total_pages
    );
    for (node, pages) in &entry.node_pages {
        println!("numa_maps_node={} pages={pages}", node.0);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mapped_scratch_bind=unsupported-platform");
}
