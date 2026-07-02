#![allow(missing_docs)]

use std::error::Error;
use std::io::ErrorKind;

use locus_observe::{read_self_numa_maps, NumaMapsSummary, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let entries = match read_self_numa_maps() {
        Ok(entries) => entries,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("numa_maps=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let summary = NumaMapsSummary::from_entries(&entries);

    println!("mappings={}", summary.mapping_count);
    println!("pages={}", summary.total_pages);
    for (node, pages) in summary.pages_by_node {
        println!("node={} pages={}", node.0, pages);
    }
    for (policy, mappings) in summary.mappings_by_policy {
        println!("policy={policy} mappings={mappings}");
    }
    for (page_kb, pages) in summary.pages_by_kernel_page_kb {
        println!("kernel_page_kb={page_kb} pages={pages}");
    }

    Ok(())
}
