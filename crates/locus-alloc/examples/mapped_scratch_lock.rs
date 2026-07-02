#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_alloc::MappedScratchArena;
    use locus_core::NodeId;

    let mut arena = MappedScratchArena::new(NodeId(0), 16 * 1024)?;
    let mapping_start = arena.mapping_start_address();
    println!("mapping_start=0x{mapping_start:x}");
    println!("mapping_len={}", arena.mapping_len());

    let touched = arena.write_touch_pages()?;
    println!("touched={touched}");

    match arena.lock_pages() {
        Ok(()) => {
            println!("page_lock=ok");
            match arena.unlock_pages() {
                Ok(()) => println!("page_unlock=ok"),
                Err(error) => {
                    println!("page_unlock=error");
                    println!("page_unlock_error={error}");
                }
            }
        }
        Err(error) => {
            println!("page_lock=error");
            println!("page_lock_error={error}");
        }
    }

    Ok(())
}
