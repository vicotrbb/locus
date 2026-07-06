#![allow(missing_docs)]
#![cfg_attr(
    not(all(feature = "numa", target_os = "linux")),
    allow(dead_code, unused_imports)
)]

use std::alloc::Layout;

use locus::Topology;
use locus::{PinnedScratchPool, PinnedScratchPoolError, PinnedScratchPoolStats};
use locus_validate::evaluate_pinned_scratch_near_gpu_validation_output;

#[cfg(all(feature = "numa", target_os = "linux"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu_arg = parse_gpu_arg()?;
    let topology = locus::topology::discovery::discover()?;
    let arena_capacity = 16 * 1024;
    let arena_mapping_len = arena_capacity + 4096 - 1;
    let max_locked_bytes = arena_mapping_len * 2;
    let mut output = String::new();

    emit_line(
        &mut output,
        format_args!("topology_nodes={}", topology.nodes.len()),
    );
    emit_line(
        &mut output,
        format_args!("topology_pci_devices={}", topology.pci_devices.len()),
    );

    let Some(gpu_bdf) = gpu_arg.or_else(|| first_gpu_with_numa_node(&topology)) else {
        emit_line(
            &mut output,
            format_args!("near_gpu_pool=unavailable reason=no_gpu_with_numa_node"),
        );
        emit_gate(&output)?;
        return Ok(());
    };

    emit_pool_config(&mut output, &gpu_bdf, arena_capacity, max_locked_bytes);
    if let Some(mut pool) = build_near_gpu_pool(
        &mut output,
        gpu_bdf,
        &topology,
        arena_capacity,
        max_locked_bytes,
    ) {
        run_pool_probe(&mut output, &mut pool)?;
    }

    emit_gate(&output)?;

    Ok(())
}

fn emit_pool_config(
    output: &mut String,
    gpu_bdf: &str,
    arena_capacity: usize,
    max_locked_bytes: usize,
) {
    emit_line(output, format_args!("gpu_bdf={gpu_bdf}"));
    emit_line(output, format_args!("arena_capacity={arena_capacity}"));
    emit_line(output, format_args!("max_locked_bytes={max_locked_bytes}"));
}

fn build_near_gpu_pool(
    output: &mut String,
    gpu_bdf: String,
    topology: &Topology,
    arena_capacity: usize,
    max_locked_bytes: usize,
) -> Option<PinnedScratchPool> {
    match PinnedScratchPool::new_near_gpu(gpu_bdf, topology, arena_capacity, max_locked_bytes) {
        Ok(pool) => {
            emit_line(
                output,
                format_args!("near_gpu_pool=ok home_node={}", pool.stats().home_node.0),
            );
            Some(pool)
        }
        Err(PinnedScratchPoolError::GpuLocalityUnavailable { reason, .. }) => {
            emit_line(
                output,
                format_args!(
                    "near_gpu_pool=unavailable reason={}",
                    near_gpu_unavailable_reason(reason)
                ),
            );
            None
        }
        Err(error) => {
            emit_line(output, format_args!("near_gpu_pool=error"));
            emit_line(output, format_args!("near_gpu_pool_error={error}"));
            None
        }
    }
}

fn run_pool_probe(
    output: &mut String,
    pool: &mut PinnedScratchPool,
) -> Result<(), Box<dyn std::error::Error>> {
    emit_stats(output, "initial", pool.stats());

    let Some(handle) = checkout_pool(output, pool) else {
        return Ok(());
    };

    allocate_from_handle(output, pool, handle)?;
    emit_stats(output, "after_checkout", pool.stats());
    release_handle(output, pool, handle);

    Ok(())
}

fn checkout_pool(
    output: &mut String,
    pool: &mut PinnedScratchPool,
) -> Option<locus::PinnedScratchHandle> {
    match pool.checkout() {
        Ok(handle) => {
            emit_line(
                output,
                format_args!("pool_checkout=ok handle={}", handle.id()),
            );
            Some(handle)
        }
        Err(error) => {
            emit_line(output, format_args!("pool_checkout=error"));
            emit_line(output, format_args!("pool_checkout_error={error}"));
            emit_stats(output, "checkout_error", pool.stats());
            None
        }
    }
}

fn allocate_from_handle(
    output: &mut String,
    pool: &mut PinnedScratchPool,
    handle: locus::PinnedScratchHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let arena = pool.get_mut(handle)?;
    emit_line(
        output,
        format_args!(
            "checked_out_mapping_start=0x{:x}",
            arena.mapping_start_address()
        ),
    );
    emit_line(
        output,
        format_args!("checked_out_mapping_len={}", arena.mapping_len()),
    );

    match arena.alloc_bytes(Layout::from_size_align(256, 64)?) {
        Ok(allocation) => {
            allocation[0] = 17;
            emit_line(
                output,
                format_args!("checked_out_allocation=ok bytes={}", allocation.len()),
            );
        }
        Err(error) => {
            emit_line(output, format_args!("checked_out_allocation=error"));
            emit_line(output, format_args!("checked_out_allocation_error={error}"));
        }
    }
    Ok(())
}

fn release_handle(
    output: &mut String,
    pool: &mut PinnedScratchPool,
    handle: locus::PinnedScratchHandle,
) {
    match pool.release(handle) {
        Ok(()) => {
            emit_line(
                output,
                format_args!("pool_release=ok handle={}", handle.id()),
            );
        }
        Err(error) => {
            emit_line(output, format_args!("pool_release=error"));
            emit_line(output, format_args!("pool_release_error={error}"));
        }
    }
    emit_stats(output, "after_release", pool.stats());
}

fn parse_gpu_arg() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "live_pinned_scratch_near_gpu_validation_gate".to_owned());
    let gpu = args.next();
    if args.next().is_some() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("usage: {program} [gpu-bdf]"),
        )));
    }
    Ok(gpu)
}

fn first_gpu_with_numa_node(topology: &Topology) -> Option<String> {
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

fn emit_gate(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gate = evaluate_pinned_scratch_near_gpu_validation_output(output)?;
    println!("{gate}");
    Ok(())
}

fn emit_line(output: &mut String, args: std::fmt::Arguments<'_>) {
    let line = args.to_string();
    println!("{line}");
    output.push_str(&line);
    output.push('\n');
}

fn emit_stats(output: &mut String, phase: &str, stats: PinnedScratchPoolStats) {
    emit_line(
        output,
        format_args!(
            "pool_stats phase={phase} locked_bytes={} checked_out={} idle={} created_arenas={} reused_arenas={} checkout_count={} release_count={}",
            stats.locked_bytes,
            stats.checked_out,
            stats.idle,
            stats.created_arenas,
            stats.reused_arenas,
            stats.checkout_count,
            stats.release_count
        ),
    );
}

#[cfg(not(all(feature = "numa", target_os = "linux")))]
fn main() {
    println!("near_gpu=unavailable reason=requires_linux_and_numa_feature");
}
