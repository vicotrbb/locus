#![allow(missing_docs)]

use std::error::Error;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{read_cgroup_numa_stat, CgroupNumaSummary, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("/sys/fs/cgroup/memory.numa_stat");
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
