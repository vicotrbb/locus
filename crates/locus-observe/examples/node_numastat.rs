#![allow(missing_docs)]

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{read_node_numastat, NodeNumastatSnapshot, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let node_root = Path::new("/sys/devices/system/node");
    if !node_root.exists() {
        println!("node_numastat=unavailable");
        return Ok(());
    }

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

    if node_paths.is_empty() {
        println!("node_numastat=unavailable");
        return Ok(());
    }

    for node_path in node_paths {
        let node_name = node_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        let stat_path = node_path.join("numastat");
        let metrics = match read_node_numastat(&stat_path) {
            Ok(metrics) => metrics,
            Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
                println!("node={node_name} numastat=unavailable");
                continue;
            }
            Err(error) => return Err(Box::new(error)),
        };

        let snapshot = NodeNumastatSnapshot::from_metrics(&metrics);
        println!("node={node_name} metrics={}", snapshot.metric_count);
        for metric in ["numa_hit", "numa_miss", "local_node", "other_node"] {
            if let Some(value) = snapshot.get(metric) {
                println!("node={node_name} {metric}={value}");
            }
        }
    }

    Ok(())
}
