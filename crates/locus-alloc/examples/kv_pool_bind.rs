//! Validates NUMA binding of a mapped-region KV block pool.
//!
//! Prints one status line usable on any host; the bind step only runs on
//! Linux and reports the outcome honestly elsewhere.

use locus_alloc::{KvBlockPool, KvReuseOrder};
use locus_core::NodeId;

fn main() {
    let block_size = 4096_usize;
    let capacity = 1024_usize;
    let pool = match KvBlockPool::new_mapped(NodeId(0), block_size, capacity, KvReuseOrder::Lifo) {
        Ok(pool) => pool,
        Err(error) => {
            println!("kv_pool_bind status=mapping_failed error={error}");
            return;
        }
    };
    let (start, len) = pool.mapping_span().expect("mapped pool has a span");
    println!("kv_pool_bind mapping_start={start:#x} mapping_len={len} blocks={capacity}");

    #[cfg(target_os = "linux")]
    {
        match pool.bind_to_node(NodeId(0)) {
            Ok(()) => println!("kv_pool_bind status=bound node=0"),
            Err(error) => println!("kv_pool_bind status=bind_failed node=0 error={error}"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    println!("kv_pool_bind status=bind_unsupported_on_host");
}
