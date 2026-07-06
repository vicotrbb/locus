#![allow(missing_docs)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use locus_alloc::sys::linux::{
        bind_region_to_node, read_current_process_status_diagnostics, LinuxNumaPolicyReadiness,
    };
    use locus_alloc::sys::{page_size, MappedRegion};

    let size = page_size()?;
    let mut region = MappedRegion::anonymous(size * 4)?;

    let bind_result = bind_region_to_node(&region, 0);
    match &bind_result {
        Ok(()) => println!("mbind=ok"),
        Err(error) => println!("mbind=error {error}"),
    }
    let readiness = LinuxNumaPolicyReadiness::from_bind_result(bind_result.as_ref().map(|_| ()));
    println!("{readiness}");
    match read_current_process_status_diagnostics() {
        Ok(diagnostics) => println!("{diagnostics}"),
        Err(_) => {
            println!("seccomp=unavailable seccomp_filters=unavailable no_new_privs=unavailable")
        }
    }

    let touched = region.write_touch_pages(size)?;
    println!("touched={touched}");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("mbind=unsupported-platform");
}
