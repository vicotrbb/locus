#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::error::Error;
use std::io::ErrorKind;

use locus_core::NodeId;
use locus_observe::{read_self_numa_maps, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let entries = match read_self_numa_maps() {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let mut pages_by_node = BTreeMap::<NodeId, u64>::new();

    for entry in &entries {
        for (node, pages) in &entry.node_pages {
            *pages_by_node.entry(*node).or_default() += pages;
        }
    }

    println!("mappings={}", entries.len());
    for (node, pages) in pages_by_node {
        println!("node={} pages={}", node.0, pages);
    }

    Ok(())
}
