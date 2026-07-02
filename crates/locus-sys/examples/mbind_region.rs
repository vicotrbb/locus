#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_sys::linux::bind_region_to_node;
    use locus_sys::{page_size, MappedRegion};

    let size = page_size()?;
    let mut region = MappedRegion::anonymous(size * 4)?;

    match bind_region_to_node(&region, 0) {
        Ok(()) => println!("mbind=ok"),
        Err(error) => println!("mbind=error {error}"),
    }

    let touched = region.write_touch_pages(size)?;
    println!("touched={touched}");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mbind=unsupported-platform");
}
