#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use locus_alloc::{MappedScratchAllocError, MappedScratchArena};
    use locus_core::NodeId;
    use locus_observe::{
        numa_maps_entry_for_address, read_self_numa_maps, NumaPlacementProof,
        NumaPlacementProofReason, NumaPlacementValidationReadiness, ObserveReadError,
    };
    use locus_sys::linux::LinuxNumaPolicyReadiness;
    use locus_validate::linux::PlacementValidationGate;

    let mut arena = MappedScratchArena::new(NodeId(0), 16 * 1024)?;
    let mapping_start = arena.mapping_start_address();
    println!("mapping_start=0x{mapping_start:x}");
    println!("mapping_len={}", arena.mapping_len());

    let bind_result = arena.bind_to_node(NodeId(0));
    let policy_applied = bind_result.is_ok();
    let memory_policy = match &bind_result {
        Ok(()) => LinuxNumaPolicyReadiness::from_bind_result(Ok(())),
        Err(MappedScratchAllocError::LinuxNumaPolicy(source)) => {
            LinuxNumaPolicyReadiness::from_bind_result(Err(source))
        }
        Err(_) => unreachable!("bind_to_node returns only Linux NUMA policy errors"),
    };
    println!(
        "memory_policy_readiness={} reason={}",
        memory_policy.status, memory_policy.reason
    );

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");
    println!("home_node={}", arena.home_node().0);

    let cgroup_available = current_cgroup_summary()?.is_some();
    let node_numastat_available = current_node_numastat_snapshot()?.is_some();
    let numa_maps_result = match read_self_numa_maps() {
        Ok(entries) => Ok(entries),
        Err(error) => match &error {
            ObserveReadError::Read { source, .. } if source.kind() == ErrorKind::NotFound => {
                Err(error)
            }
            _ => return Err(Box::new(error)),
        },
    };
    let numa_maps_available = numa_maps_result.is_ok();
    let placement_readiness = NumaPlacementValidationReadiness::from_sources(
        numa_maps_available,
        cgroup_available,
        node_numastat_available,
    );

    println!(
        "numa_maps={}",
        if numa_maps_available {
            "available"
        } else {
            "unavailable"
        }
    );
    println!(
        "cgroup_numa_stat={}",
        if cgroup_available {
            "available"
        } else {
            "unavailable"
        }
    );
    println!(
        "node_numastat={}",
        if node_numastat_available {
            "available"
        } else {
            "unavailable"
        }
    );
    println!(
        "placement_validation_readiness={} reason={}",
        placement_readiness.status, placement_readiness.reason
    );

    let placement_proof = match numa_maps_result {
        Ok(entries) => {
            if let Some(address_match) = numa_maps_entry_for_address(&entries, mapping_start) {
                let evidence = locus_observe::NumaPlacementEvidence::from_entry(
                    address_match.entry,
                    arena.home_node(),
                );
                NumaPlacementProof::from_evidence(policy_applied, Some(&evidence))
            } else {
                NumaPlacementProof::from_evidence(policy_applied, None)
            }
        }
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            NumaPlacementProof::unavailable(NumaPlacementProofReason::NumaMapsUnavailable)
        }
        Err(error) => return Err(Box::new(error)),
    };
    println!(
        "placement_proof={} reason={}",
        placement_proof.status, placement_proof.reason
    );

    let gate =
        PlacementValidationGate::from_verdicts(memory_policy, placement_readiness, placement_proof);
    println!(
        "placement_validation_gate={} reason={}",
        gate.status, gate.reason
    );

    Ok(())
}

#[cfg(target_os = "linux")]
fn current_node_numastat_snapshot(
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
fn current_cgroup_summary(
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

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("placement_validation_gate=unsupported-platform");
}
