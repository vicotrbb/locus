#![allow(missing_docs)]

use std::alloc::Layout;

use locus_alloc::NodeId;
use locus_alloc::{PinnedScratchPool, PinnedScratchPoolStats};
use locus_validate::evaluate_pinned_scratch_validation_output;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arena_capacity = 16 * 1024;
    let arena_mapping_len = arena_capacity + 4096 - 1;
    let max_locked_bytes = arena_mapping_len * 2;
    let mut pool = PinnedScratchPool::new(NodeId(0), arena_capacity, max_locked_bytes)?;
    let mut output = String::new();

    emit_line(&mut output, format_args!("arena_capacity={arena_capacity}"));
    emit_line(
        &mut output,
        format_args!("max_locked_bytes={max_locked_bytes}"),
    );
    emit_stats(&mut output, "initial", pool.stats());

    let first = match pool.checkout() {
        Ok(handle) => {
            emit_line(
                &mut output,
                format_args!("pool_checkout=ok handle={}", handle.id()),
            );
            handle
        }
        Err(error) => {
            emit_line(&mut output, format_args!("pool_checkout=error"));
            emit_line(&mut output, format_args!("pool_checkout_error={error}"));
            emit_stats(&mut output, "checkout_error", pool.stats());
            let gate = evaluate_pinned_scratch_validation_output(&output)?;
            println!("{gate}");
            return Ok(());
        }
    };

    {
        let arena = pool.get_mut(first)?;
        emit_line(
            &mut output,
            format_args!(
                "checked_out_mapping_start=0x{:x}",
                arena.mapping_start_address()
            ),
        );
        emit_line(
            &mut output,
            format_args!("checked_out_mapping_len={}", arena.mapping_len()),
        );
        let allocation = arena.alloc_bytes(Layout::from_size_align(256, 64)?)?;
        allocation[0] = 11;
        emit_line(
            &mut output,
            format_args!("checked_out_allocation=ok bytes={}", allocation.len()),
        );
    }
    emit_stats(&mut output, "after_checkout", pool.stats());

    pool.release(first)?;
    emit_line(
        &mut output,
        format_args!("pool_release=ok handle={}", first.id()),
    );
    emit_stats(&mut output, "after_release", pool.stats());

    let second = pool.checkout()?;
    emit_line(
        &mut output,
        format_args!("pool_reuse_checkout=ok handle={}", second.id()),
    );
    emit_stats(&mut output, "after_reuse_checkout", pool.stats());

    pool.release(second)?;
    emit_line(
        &mut output,
        format_args!("pool_reuse_release=ok handle={}", second.id()),
    );
    emit_stats(&mut output, "after_reuse_release", pool.stats());

    let gate = evaluate_pinned_scratch_validation_output(&output)?;
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
