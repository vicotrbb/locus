#![allow(missing_docs)]

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{
    read_cgroup_numa_stat, resolve_cgroup_v2_memory_numa_stat_path, CgroupNumaSummary,
    ObserveReadError,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cgroup_content = fs::read_to_string("/proc/self/cgroup")?;
    let path =
        resolve_cgroup_v2_memory_numa_stat_path(&cgroup_content, Path::new("/sys/fs/cgroup"))?;
    let entries = match read_cgroup_numa_stat(path) {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("cgroup_numa_stat=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let summary = CgroupNumaSummary::from_entries(&entries);
    println!("metrics={}", summary.metric_count);
    println!("bytes={}", summary.total_bytes);
    for (node, bytes) in summary.bytes_by_node {
        println!("node={} bytes={bytes}", node.0);
    }

    Ok(())
}
