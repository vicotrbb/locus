#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use locus_alloc::MappedScratchArena;
    use locus_core::NodeId;
    use locus_observe::{numa_maps_entry_for_address, read_self_numa_maps, ObserveReadError};

    let mut arena = MappedScratchArena::new(NodeId(0), 16 * 1024)?;
    let mapping_start = arena.mapping_start_address();

    println!("mapping_start=0x{mapping_start:x}");
    println!("mapping_len={}", arena.mapping_len());

    let cgroup_before = read_current_cgroup_summary()?;
    let node_numastat_before = read_current_node_numastat_snapshot()?;

    match arena.bind_to_node(NodeId(0)) {
        Ok(()) => println!("mapped_scratch_bind=ok"),
        Err(error) => println!("mapped_scratch_bind=error {error}"),
    }

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");
    println!("home_node={}", arena.home_node().0);

    match cgroup_before {
        Some(before) => match read_current_cgroup_summary()? {
            Some(after) => print_cgroup_delta(&before, &after),
            None => println!("cgroup_numa_delta=unavailable"),
        },
        None => println!("cgroup_numa_delta=unavailable"),
    }

    match node_numastat_before {
        Some(before) => match read_current_node_numastat_snapshot()? {
            Some(after) => print_node_numastat_delta(&before, &after),
            None => println!("node_numastat_delta=unavailable"),
        },
        None => println!("node_numastat_delta=unavailable"),
    }

    match read_self_numa_maps() {
        Ok(entries) => {
            if let Some(address_match) = numa_maps_entry_for_address(&entries, mapping_start) {
                print_placement(address_match, arena.home_node());
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
    address_match: locus_observe::NumaMapsAddressMatch<'_>,
    expected_node: locus_core::NodeId,
) {
    let entry = address_match.entry;
    let evidence = locus_observe::NumaPlacementEvidence::from_entry(entry, expected_node);
    println!(
        "numa_maps_match={} policy={} placement_status={} placement_verified={} expected_node={} expected_pages={} other_pages={} total_pages={}",
        address_match.kind,
        entry.policy,
        evidence.status,
        evidence.is_fully_on_expected_node(),
        evidence.expected_node.0,
        evidence.expected_node_pages,
        evidence.other_pages(),
        evidence.total_pages
    );
    for (node, pages) in &entry.node_pages {
        println!("numa_maps_node={} pages={pages}", node.0);
    }
}

#[cfg(target_os = "linux")]
fn read_current_node_numastat_snapshot(
) -> Result<Option<locus_observe::NodeNumastatSystemSnapshot>, Box<dyn std::error::Error>> {
    match locus_observe::read_node_numastat_system_snapshot(std::path::Path::new(
        "/sys/devices/system/node",
    )) {
        Ok(snapshot) if snapshot.node_count == 0 => Ok(None),
        Ok(snapshot) => Ok(Some(snapshot)),
        Err(locus_observe::ObserveReadError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(Box::new(error)),
    }
}

#[cfg(target_os = "linux")]
fn read_current_cgroup_summary(
) -> Result<Option<locus_observe::CgroupNumaSummary>, Box<dyn std::error::Error>> {
    let cgroup_content = match std::fs::read_to_string("/proc/self/cgroup") {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Box::new(error)),
    };
    let path = match locus_observe::resolve_cgroup_v2_memory_numa_stat_path(
        &cgroup_content,
        std::path::Path::new("/sys/fs/cgroup"),
    ) {
        Ok(path) => path,
        Err(locus_observe::CgroupPathError::MissingUnifiedEntry) => return Ok(None),
    };

    match locus_observe::read_cgroup_numa_summary(path) {
        Ok(summary) => Ok(Some(summary)),
        Err(locus_observe::ObserveReadError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(Box::new(error)),
    }
}

#[cfg(target_os = "linux")]
fn print_node_numastat_delta(
    before: &locus_observe::NodeNumastatSystemSnapshot,
    after: &locus_observe::NodeNumastatSystemSnapshot,
) {
    let delta = after.delta_since(before);
    println!("node_numastat_delta=ok nodes={}", delta.nodes.len());
    for (node, node_delta) in delta.nodes {
        for metric in ["numa_hit", "numa_miss", "local_node", "other_node"] {
            if let Some(value) = node_delta.get(metric) {
                println!("node_numastat_node={} {metric}_delta={value}", node.0);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn print_cgroup_delta(
    before: &locus_observe::CgroupNumaSummary,
    after: &locus_observe::CgroupNumaSummary,
) {
    let delta = after.delta_since(before);
    println!(
        "cgroup_numa_delta=ok total_bytes_delta={}",
        delta.total_bytes_delta
    );
    for (node, bytes_delta) in delta.bytes_by_node_delta {
        println!("cgroup_numa_node={} bytes_delta={bytes_delta}", node.0);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mapped_scratch_bind=unsupported-platform");
}
