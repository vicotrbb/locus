#![allow(missing_docs)]

use std::error::Error;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{
    read_cgroup_numa_stat, read_node_numastat, read_self_numa_maps,
    resolve_cgroup_v2_memory_numa_stat_path, CgroupNumaSummary, CgroupPathError,
    NodeNumastatSnapshot, NumaMapsSummary, ObserveReadError,
};

fn main() -> Result<(), Box<dyn Error>> {
    probe_self_numa_maps()?;
    probe_cgroup_numa_stat()?;
    probe_node_numastat()?;
    Ok(())
}

fn probe_self_numa_maps() -> Result<(), Box<dyn Error>> {
    let entries = match read_self_numa_maps() {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let summary = NumaMapsSummary::from_entries(&entries);
    println!(
        "numa_maps=available mappings={} pages={}",
        summary.mapping_count, summary.total_pages
    );
    Ok(())
}

fn probe_cgroup_numa_stat() -> Result<(), Box<dyn Error>> {
    let cgroup_content = match fs::read_to_string("/proc/self/cgroup") {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            println!("cgroup_numa_stat=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };
    let path =
        match resolve_cgroup_v2_memory_numa_stat_path(&cgroup_content, Path::new("/sys/fs/cgroup"))
        {
            Ok(path) => path,
            Err(CgroupPathError::MissingUnifiedEntry) => {
                println!("cgroup_numa_stat=unavailable");
                return Ok(());
            }
        };

    let entries = match read_cgroup_numa_stat(path) {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("cgroup_numa_stat=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let summary = CgroupNumaSummary::from_entries(&entries);
    println!(
        "cgroup_numa_stat=available metrics={} bytes={}",
        summary.metric_count, summary.total_bytes
    );
    Ok(())
}

fn probe_node_numastat() -> Result<(), Box<dyn Error>> {
    let node_root = Path::new("/sys/devices/system/node");
    if !node_root.exists() {
        println!("node_numastat=unavailable");
        return Ok(());
    }

    let node_paths = node_paths(node_root)?;
    if node_paths.is_empty() {
        println!("node_numastat=unavailable");
        return Ok(());
    }

    let mut available_nodes = 0_usize;
    let mut metric_count = 0_usize;
    for node_path in node_paths {
        let stat_path = node_path.join("numastat");
        let metrics = match read_node_numastat(&stat_path) {
            Ok(metrics) => metrics,
            Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(Box::new(error)),
        };

        let snapshot = NodeNumastatSnapshot::from_metrics(&metrics);
        available_nodes += 1;
        metric_count += snapshot.metric_count;
    }

    if available_nodes == 0 {
        println!("node_numastat=unavailable");
    } else {
        println!("node_numastat=available nodes={available_nodes} metrics={metric_count}");
    }

    Ok(())
}

fn node_paths(node_root: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let mut node_paths = Vec::new();
    for entry in fs::read_dir(node_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.strip_prefix("node").is_some() {
            node_paths.push(entry.path());
        }
    }
    node_paths.sort();
    Ok(node_paths)
}
