#![allow(missing_docs)]

use std::alloc::Layout;

use locus_alloc::{PinnedScratchPool, PinnedScratchPoolError, PinnedScratchPoolStats};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu_arg = parse_gpu_arg()?;
    let topology = locus_topology::discover()?;
    let arena_capacity = 16 * 1024;
    let arena_mapping_len = arena_capacity + 4096 - 1;
    let max_locked_bytes = arena_mapping_len * 2;

    println!("topology_nodes={}", topology.nodes.len());
    println!("topology_pci_devices={}", topology.pci_devices.len());

    let Some(gpu_bdf) = gpu_arg.or_else(|| first_gpu_with_numa_node(&topology)) else {
        println!("near_gpu_pool=unavailable reason=no_gpu_with_numa_node");
        return Ok(());
    };

    println!("gpu_bdf={gpu_bdf}");
    println!("arena_capacity={arena_capacity}");
    println!("max_locked_bytes={max_locked_bytes}");

    let mut pool = match PinnedScratchPool::new_near_gpu(
        gpu_bdf.clone(),
        &topology,
        arena_capacity,
        max_locked_bytes,
    ) {
        Ok(pool) => {
            println!("near_gpu_pool=ok home_node={}", pool.stats().home_node.0);
            pool
        }
        Err(PinnedScratchPoolError::GpuLocalityUnavailable { reason, .. }) => {
            println!(
                "near_gpu_pool=unavailable reason={}",
                near_gpu_unavailable_reason(reason)
            );
            return Ok(());
        }
        Err(error) => {
            println!("near_gpu_pool=error");
            println!("near_gpu_pool_error={error}");
            return Ok(());
        }
    };

    print_stats("initial", pool.stats());

    let handle = match pool.checkout() {
        Ok(handle) => {
            println!("pool_checkout=ok handle={}", handle.id());
            handle
        }
        Err(error) => {
            println!("pool_checkout=error");
            println!("pool_checkout_error={error}");
            print_stats("checkout_error", pool.stats());
            return Ok(());
        }
    };

    {
        let arena = pool.get_mut(handle)?;
        println!(
            "checked_out_mapping_start=0x{:x}",
            arena.mapping_start_address()
        );
        println!("checked_out_mapping_len={}", arena.mapping_len());
        let allocation = arena.alloc_bytes(Layout::from_size_align(256, 64)?)?;
        allocation[0] = 13;
        println!("checked_out_allocation=ok bytes={}", allocation.len());
    }
    print_stats("after_checkout", pool.stats());

    pool.release(handle)?;
    println!("pool_release=ok handle={}", handle.id());
    print_stats("after_release", pool.stats());

    Ok(())
}

fn parse_gpu_arg() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "pinned_scratch_near_gpu".to_owned());
    let gpu = args.next();
    if args.next().is_some() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("usage: {program} [gpu-bdf]"),
        )));
    }
    Ok(gpu)
}

fn first_gpu_with_numa_node(topology: &locus_core::Topology) -> Option<String> {
    topology
        .pci_devices
        .iter()
        .find(|device| device.numa_node.is_some())
        .map(|device| device.bdf.clone())
}

fn near_gpu_unavailable_reason(reason: &str) -> &'static str {
    match reason {
        "GPU PCI device was not discovered, using local first-touch behavior" => "gpu_missing",
        "GPU PCI device has no reported NUMA node, using local first-touch behavior" => {
            "gpu_numa_node_unavailable"
        }
        "GPU locality resolved to an empty node set" => "empty_node_set",
        "GPU locality resolved to multiple NUMA nodes" => "multiple_nodes",
        _ => "unresolved",
    }
}

fn print_stats(phase: &str, stats: PinnedScratchPoolStats) {
    println!(
        "pool_stats phase={phase} locked_bytes={} checked_out={} idle={} created_arenas={} reused_arenas={} checkout_count={} release_count={}",
        stats.locked_bytes,
        stats.checked_out,
        stats.idle,
        stats.created_arenas,
        stats.reused_arenas,
        stats.checkout_count,
        stats.release_count
    );
}
