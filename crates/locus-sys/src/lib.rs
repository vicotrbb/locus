//! Narrow system boundary for Locus memory primitives.
//!
//! This crate is the intended home for operating-system calls and raw memory
//! handles. Public APIs should stay safe and owned where possible.

use std::fmt;
use std::io;
use std::ptr::NonNull;
use std::slice;

/// Owned anonymous memory mapping.
#[derive(Debug)]
pub struct MappedRegion {
    ptr: NonNull<u8>,
    len: usize,
}

impl MappedRegion {
    /// Creates a private anonymous read-write mapping.
    ///
    /// # Errors
    ///
    /// Returns an error when `len` is zero or the operating system rejects the
    /// mapping request.
    pub fn anonymous(len: usize) -> Result<Self, MappedRegionError> {
        if len == 0 {
            return Err(MappedRegionError::InvalidLength);
        }

        let raw = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };

        if raw == libc::MAP_FAILED {
            return Err(MappedRegionError::MapFailed(io::Error::last_os_error()));
        }

        let ptr = NonNull::new(raw.cast::<u8>())
            .ok_or_else(|| MappedRegionError::MapFailed(io::Error::last_os_error()))?;
        Ok(Self { ptr, len })
    }

    /// Returns the mapping length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true when the mapping is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the mapping as a shared byte slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns the mapping as an exclusive byte slice.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        let rc = unsafe { libc::munmap(self.ptr.as_ptr().cast::<libc::c_void>(), self.len) };
        debug_assert_eq!(rc, 0, "munmap failed: {}", io::Error::last_os_error());
    }
}

/// Mapping failures from the system boundary.
#[derive(Debug)]
pub enum MappedRegionError {
    /// Mapping length must be non-zero.
    InvalidLength,
    /// The operating system rejected the mapping.
    MapFailed(io::Error),
}

impl fmt::Display for MappedRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("mapped region length must be non-zero"),
            Self::MapFailed(source) => write!(f, "anonymous mmap failed: {source}"),
        }
    }
}

impl std::error::Error for MappedRegionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidLength => None,
            Self::MapFailed(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MappedRegion, MappedRegionError};

    #[test]
    fn maps_writable_anonymous_region() {
        let mut region = MappedRegion::anonymous(4096).expect("mapped region");

        assert_eq!(region.len(), 4096);
        assert!(!region.is_empty());
        region.as_mut_slice()[0] = 7;
        region.as_mut_slice()[4095] = 9;

        assert_eq!(region.as_slice()[0], 7);
        assert_eq!(region.as_slice()[4095], 9);
    }

    #[test]
    fn rejects_zero_length_mapping() {
        let error = MappedRegion::anonymous(0).expect_err("zero length should fail");

        assert!(matches!(error, MappedRegionError::InvalidLength));
    }
}
