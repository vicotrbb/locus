#![allow(missing_docs)]

use std::error::Error;
use std::io::ErrorKind;

use locus_observe::{read_self_process_fault_counts, ObserveReadError};

fn main() -> Result<(), Box<dyn Error>> {
    let counts = match read_self_process_fault_counts() {
        Ok(counts) => counts,
        Err(ObserveReadError::Read { source, .. }) if source.kind() == ErrorKind::NotFound => {
            println!("process_faults=unavailable");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    println!(
        "process_faults=available minor_faults={} child_minor_faults={} major_faults={} child_major_faults={}",
        counts.minor_faults,
        counts.child_minor_faults,
        counts.major_faults,
        counts.child_major_faults
    );

    Ok(())
}
