#![allow(missing_docs)]

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{
    read_cgroup_numa_summary, read_node_numastat_system_snapshot, read_self_numa_maps,
    resolve_cgroup_v2_memory_numa_stat_path, CgroupPathError, NumaMapsSummary,
    NumaPlacementValidationReadiness, ObserveReadError,
};

fn main() -> Result<(), Box<dyn Error>> {
    let numa_maps_available = probe_self_numa_maps()?;
    let cgroup_numa_stat_available = probe_cgroup_numa_stat()?;
    let node_numastat_available = probe_node_numastat()?;
    let readiness = NumaPlacementValidationReadiness::from_sources(
        numa_maps_available,
        cgroup_numa_stat_available,
        node_numastat_available,
    );
    println!("{readiness}");
    Ok(())
}

fn probe_self_numa_maps() -> Result<bool, Box<dyn Error>> {
    let entries = match read_self_numa_maps() {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
            return Ok(false);
        }
        Err(error) => return Err(Box::new(error)),
    };

    let summary = NumaMapsSummary::from_entries(&entries);
    println!(
        "numa_maps=available mappings={} pages={}",
        summary.mapping_count, summary.total_pages
    );
    Ok(true)
}

fn probe_cgroup_numa_stat() -> Result<bool, Box<dyn Error>> {
    let cgroup_content = match fs::read_to_string("/proc/self/cgroup") {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            println!("cgroup_numa_stat=unavailable");
            return Ok(false);
        }
        Err(error) => return Err(Box::new(error)),
    };
    let path =
        match resolve_cgroup_v2_memory_numa_stat_path(&cgroup_content, Path::new("/sys/fs/cgroup"))
        {
            Ok(path) => path,
            Err(CgroupPathError::MissingUnifiedEntry) => {
                println!("cgroup_numa_stat=unavailable");
                return Ok(false);
            }
        };

    let summary = match read_cgroup_numa_summary(path) {
        Ok(summary) => summary,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("cgroup_numa_stat=unavailable");
            return Ok(false);
        }
        Err(error) => return Err(Box::new(error)),
    };

    println!(
        "cgroup_numa_stat=available metrics={} bytes={}",
        summary.metric_count, summary.total_bytes
    );
    Ok(true)
}

fn probe_node_numastat() -> Result<bool, Box<dyn Error>> {
    let node_root = Path::new("/sys/devices/system/node");
    let snapshot = match read_node_numastat_system_snapshot(node_root) {
        Ok(snapshot) => snapshot,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("node_numastat=unavailable");
            return Ok(false);
        }
        Err(error) => return Err(Box::new(error)),
    };

    if snapshot.node_count == 0 {
        println!("node_numastat=unavailable");
        Ok(false)
    } else {
        let metric_count = snapshot
            .nodes
            .values()
            .map(|node| node.metric_count)
            .sum::<usize>();
        println!(
            "node_numastat=available nodes={} metrics={metric_count}",
            snapshot.node_count
        );
        Ok(true)
    }
}
