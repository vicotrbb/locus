#![allow(missing_docs)]

use std::alloc::Layout;

use locus_alloc::NodeId;
use locus_alloc::{PinnedScratchPool, PinnedScratchPoolStats};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arena_capacity = 16 * 1024;
    let arena_mapping_len = arena_capacity + 4096 - 1;
    let max_locked_bytes = arena_mapping_len * 2;
    let mut pool = PinnedScratchPool::new(NodeId(0), arena_capacity, max_locked_bytes)?;

    println!("arena_capacity={arena_capacity}");
    println!("max_locked_bytes={max_locked_bytes}");
    print_stats("initial", pool.stats());

    let first = match pool.checkout() {
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
        let arena = pool.get_mut(first)?;
        println!(
            "checked_out_mapping_start=0x{:x}",
            arena.mapping_start_address()
        );
        println!("checked_out_mapping_len={}", arena.mapping_len());
        let allocation = arena.alloc_bytes(Layout::from_size_align(256, 64)?)?;
        allocation[0] = 11;
        println!("checked_out_allocation=ok bytes={}", allocation.len());
    }
    print_stats("after_checkout", pool.stats());

    pool.release(first)?;
    println!("pool_release=ok handle={}", first.id());
    print_stats("after_release", pool.stats());

    let second = pool.checkout()?;
    println!("pool_reuse_checkout=ok handle={}", second.id());
    print_stats("after_reuse_checkout", pool.stats());

    pool.release(second)?;
    println!("pool_reuse_release=ok handle={}", second.id());
    print_stats("after_reuse_release", pool.stats());

    Ok(())
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
