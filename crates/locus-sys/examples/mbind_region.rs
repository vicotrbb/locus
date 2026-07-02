#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_sys::linux::{bind_region_to_node, LinuxNumaPolicyReadiness};
    use locus_sys::{page_size, MappedRegion};

    let size = page_size()?;
    let mut region = MappedRegion::anonymous(size * 4)?;

    let bind_result = bind_region_to_node(&region, 0);
    match &bind_result {
        Ok(()) => println!("mbind=ok"),
        Err(error) => println!("mbind=error {error}"),
    }
    let readiness = LinuxNumaPolicyReadiness::from_bind_result(bind_result.as_ref().map(|_| ()));
    println!(
        "memory_policy_readiness={} reason={}",
        readiness.status, readiness.reason
    );

    let touched = region.write_touch_pages(size)?;
    println!("touched={touched}");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mbind=unsupported-platform");
}
