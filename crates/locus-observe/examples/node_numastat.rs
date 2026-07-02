#![allow(missing_docs)]

use std::error::Error;
use std::io::ErrorKind;
use std::path::Path;

use locus_observe::{read_node_numastat_system_snapshot, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let node_root = Path::new("/sys/devices/system/node");
    let snapshot = match read_node_numastat_system_snapshot(node_root) {
        Ok(snapshot) => snapshot,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("node_numastat=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    if snapshot.node_count == 0 {
        println!("node_numastat=unavailable");
        return Ok(());
    }

    for (node, snapshot) in snapshot.nodes {
        println!("node=node{} metrics={}", node.0, snapshot.metric_count);
        for metric in ["numa_hit", "numa_miss", "local_node", "other_node"] {
            if let Some(value) = snapshot.get(metric) {
                println!("node=node{} {metric}={value}", node.0);
            }
        }
    }

    Ok(())
}
