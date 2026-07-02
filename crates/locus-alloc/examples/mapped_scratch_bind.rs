#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_alloc::MappedScratchArena;
    use locus_core::NodeId;

    let mut arena = MappedScratchArena::new(NodeId(0), 16 * 1024)?;

    match arena.bind_to_node(NodeId(0)) {
        Ok(()) => println!("mapped_scratch_bind=ok"),
        Err(error) => println!("mapped_scratch_bind=error {error}"),
    }

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");
    println!("home_node={}", arena.home_node().0);

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mapped_scratch_bind=unsupported-platform");
}
