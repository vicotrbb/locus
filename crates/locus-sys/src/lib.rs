//! Narrow system boundary for Locus memory primitives.
//!
//! This crate is the intended home for operating-system calls and raw memory
//! handles. Public APIs should stay safe and owned where possible.

use std::fmt;
use std::io;
use std::ptr::NonNull;
use std::slice;

#[cfg(target_os = "linux")]
pub mod linux {
    //! Linux-specific NUMA memory policy helpers.

    use std::fmt;
    use std::io;
    use std::io::ErrorKind;

    use super::MappedRegion;

    const MPOL_BIND: libc::c_int = 2;
    const MAX_NODE_BITS: u32 = 4096;

    /// Applies `MPOL_BIND` for a mapped region to a single NUMA node.
    ///
    /// # Errors
    ///
    /// Returns an error when the node is outside the supported mask range or
    /// the `mbind` syscall fails.
    pub fn bind_region_to_node(
        region: &MappedRegion,
        node: u32,
    ) -> Result<(), LinuxNumaPolicyError> {
        let mask = node_mask_words(node)?;
        let max_node = libc::c_ulong::from(node) + 1;
        let rc = unsafe {
            libc::syscall(
                libc::SYS_mbind,
                region.as_ptr().cast::<libc::c_void>(),
                region.len(),
                MPOL_BIND,
                mask.as_ptr(),
                max_node,
                0,
            )
        };

        if rc == -1 {
            Err(LinuxNumaPolicyError::Syscall(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    /// Linux NUMA policy failures.
    #[derive(Debug)]
    pub enum LinuxNumaPolicyError {
        /// Node identifier is outside the supported mask range.
        InvalidNode(u32),
        /// `mbind` failed.
        Syscall(io::Error),
    }

    impl fmt::Display for LinuxNumaPolicyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidNode(node) => write!(f, "invalid NUMA node for mbind mask: {node}"),
                Self::Syscall(source) => write!(f, "mbind syscall failed: {source}"),
            }
        }
    }

    impl std::error::Error for LinuxNumaPolicyError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::InvalidNode(_) => None,
                Self::Syscall(source) => Some(source),
            }
        }
    }

    /// Readiness status for Linux NUMA memory policy application.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LinuxNumaPolicyReadinessStatus {
        /// A test memory policy application succeeded.
        Ready,
        /// A test memory policy application did not succeed.
        NotReady,
    }

    impl LinuxNumaPolicyReadinessStatus {
        /// Returns a stable machine-readable readiness status string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Ready => "ready",
                Self::NotReady => "not_ready",
            }
        }

        /// Parses a stable machine-readable readiness status string.
        #[must_use]
        pub fn from_str_token(value: &str) -> Option<Self> {
            match value {
                "ready" => Some(Self::Ready),
                "not_ready" => Some(Self::NotReady),
                _ => None,
            }
        }
    }

    impl fmt::Display for LinuxNumaPolicyReadinessStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Reason for Linux NUMA memory policy readiness.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LinuxNumaPolicyReadinessReason {
        /// A test memory policy application succeeded.
        Ready,
        /// The requested NUMA node cannot be represented in the policy mask.
        InvalidNode,
        /// The operating system denied the `mbind` syscall.
        PermissionDenied,
        /// The `mbind` syscall failed for another reason.
        SyscallFailed,
    }

    impl LinuxNumaPolicyReadinessReason {
        /// Returns a stable machine-readable readiness reason string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Ready => "ready",
                Self::InvalidNode => "invalid_node",
                Self::PermissionDenied => "permission_denied",
                Self::SyscallFailed => "syscall_failed",
            }
        }

        /// Parses a stable machine-readable readiness reason string.
        #[must_use]
        pub fn from_str_token(value: &str) -> Option<Self> {
            match value {
                "ready" => Some(Self::Ready),
                "invalid_node" => Some(Self::InvalidNode),
                "permission_denied" => Some(Self::PermissionDenied),
                "syscall_failed" => Some(Self::SyscallFailed),
                _ => None,
            }
        }
    }

    impl fmt::Display for LinuxNumaPolicyReadinessReason {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Readiness verdict for Linux NUMA memory policy application.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LinuxNumaPolicyReadiness {
        /// Final readiness status.
        pub status: LinuxNumaPolicyReadinessStatus,
        /// Reason for the status.
        pub reason: LinuxNumaPolicyReadinessReason,
    }

    /// Error returned when parsing a Linux memory-policy readiness line.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LinuxNumaPolicyReadinessLineParseError {
        /// The line does not contain a `memory_policy_readiness=` token.
        MissingStatus,
        /// The line does not contain a `reason=` token.
        MissingReason,
        /// The line contains a duplicate `memory_policy_readiness=` token.
        DuplicateStatus,
        /// The line contains a duplicate `reason=` token.
        DuplicateReason,
        /// The line contains a token outside the memory-policy readiness schema.
        InvalidToken(String),
        /// The readiness status token is not recognized.
        UnknownStatus(String),
        /// The readiness reason token is not recognized.
        UnknownReason(String),
    }

    impl fmt::Display for LinuxNumaPolicyReadinessLineParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MissingStatus => f.write_str("missing memory_policy_readiness token"),
                Self::MissingReason => f.write_str("missing reason token"),
                Self::DuplicateStatus => f.write_str("duplicate memory_policy_readiness token"),
                Self::DuplicateReason => f.write_str("duplicate reason token"),
                Self::InvalidToken(token) => {
                    write!(f, "invalid memory policy readiness token: {token}")
                }
                Self::UnknownStatus(status) => {
                    write!(f, "unknown memory policy readiness status: {status}")
                }
                Self::UnknownReason(reason) => {
                    write!(f, "unknown memory policy readiness reason: {reason}")
                }
            }
        }
    }

    impl std::error::Error for LinuxNumaPolicyReadinessLineParseError {}

    impl LinuxNumaPolicyReadiness {
        /// Builds a readiness verdict from an `mbind` attempt result.
        #[must_use]
        pub fn from_bind_result(result: Result<(), &LinuxNumaPolicyError>) -> Self {
            match result {
                Ok(()) => Self {
                    status: LinuxNumaPolicyReadinessStatus::Ready,
                    reason: LinuxNumaPolicyReadinessReason::Ready,
                },
                Err(LinuxNumaPolicyError::InvalidNode(_)) => Self {
                    status: LinuxNumaPolicyReadinessStatus::NotReady,
                    reason: LinuxNumaPolicyReadinessReason::InvalidNode,
                },
                Err(LinuxNumaPolicyError::Syscall(source))
                    if source.kind() == ErrorKind::PermissionDenied =>
                {
                    Self {
                        status: LinuxNumaPolicyReadinessStatus::NotReady,
                        reason: LinuxNumaPolicyReadinessReason::PermissionDenied,
                    }
                }
                Err(LinuxNumaPolicyError::Syscall(_)) => Self {
                    status: LinuxNumaPolicyReadinessStatus::NotReady,
                    reason: LinuxNumaPolicyReadinessReason::SyscallFailed,
                },
            }
        }

        /// Returns true only when a memory policy attempt succeeded.
        #[must_use]
        pub fn is_ready(self) -> bool {
            self.status == LinuxNumaPolicyReadinessStatus::Ready
        }
    }

    /// Parses a Linux memory-policy readiness line.
    ///
    /// The expected format is `memory_policy_readiness=<status> reason=<reason>`.
    ///
    /// # Errors
    ///
    /// Returns an error when the line is missing required tokens, contains duplicate
    /// tokens, contains unsupported tokens, or uses an unknown status or reason.
    pub fn parse_linux_numa_policy_readiness_line(
        line: &str,
    ) -> Result<LinuxNumaPolicyReadiness, LinuxNumaPolicyReadinessLineParseError> {
        let mut status_token = None;
        let mut reason_token = None;

        for token in line.split_whitespace() {
            let Some((key, value)) = token.split_once('=') else {
                return Err(LinuxNumaPolicyReadinessLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            };

            match key {
                "memory_policy_readiness" => {
                    if status_token.replace(value).is_some() {
                        return Err(LinuxNumaPolicyReadinessLineParseError::DuplicateStatus);
                    }
                }
                "reason" => {
                    if reason_token.replace(value).is_some() {
                        return Err(LinuxNumaPolicyReadinessLineParseError::DuplicateReason);
                    }
                }
                _ => {
                    return Err(LinuxNumaPolicyReadinessLineParseError::InvalidToken(
                        token.to_owned(),
                    ));
                }
            }
        }

        let status_token =
            status_token.ok_or(LinuxNumaPolicyReadinessLineParseError::MissingStatus)?;
        let reason_token =
            reason_token.ok_or(LinuxNumaPolicyReadinessLineParseError::MissingReason)?;

        let status =
            LinuxNumaPolicyReadinessStatus::from_str_token(status_token).ok_or_else(|| {
                LinuxNumaPolicyReadinessLineParseError::UnknownStatus(status_token.to_owned())
            })?;
        let reason =
            LinuxNumaPolicyReadinessReason::from_str_token(reason_token).ok_or_else(|| {
                LinuxNumaPolicyReadinessLineParseError::UnknownReason(reason_token.to_owned())
            })?;

        Ok(LinuxNumaPolicyReadiness { status, reason })
    }

    fn node_mask_words(node: u32) -> Result<Vec<libc::c_ulong>, LinuxNumaPolicyError> {
        if node >= MAX_NODE_BITS {
            return Err(LinuxNumaPolicyError::InvalidNode(node));
        }

        let word_bits = libc::c_ulong::BITS;
        let word_index = (node / word_bits) as usize;
        let bit_index = node % word_bits;
        let mut words = vec![0; word_index + 1];
        words[word_index] = libc::c_ulong::from(1_u8) << bit_index;
        Ok(words)
    }

    #[cfg(test)]
    mod tests {
        use std::io;
        use std::io::ErrorKind;

        use super::{
            node_mask_words, parse_linux_numa_policy_readiness_line, LinuxNumaPolicyError,
            LinuxNumaPolicyReadiness, LinuxNumaPolicyReadinessLineParseError,
            LinuxNumaPolicyReadinessReason, LinuxNumaPolicyReadinessStatus,
        };

        #[test]
        fn builds_single_word_node_mask() {
            let mask = node_mask_words(3).expect("node mask");

            assert_eq!(mask, vec![8]);
        }

        #[test]
        fn builds_multi_word_node_mask() {
            let node = libc::c_ulong::BITS + 1;
            let mask = node_mask_words(node).expect("node mask");

            assert_eq!(mask.len(), 2);
            assert_eq!(mask[0], 0);
            assert_eq!(mask[1], 2);
        }

        #[test]
        fn rejects_oversized_node_mask() {
            let error = node_mask_words(4096).expect_err("invalid node");

            assert!(matches!(error, LinuxNumaPolicyError::InvalidNode(4096)));
        }

        #[test]
        fn reports_linux_numa_policy_readiness() {
            let ready = LinuxNumaPolicyReadiness::from_bind_result(Ok(()));
            assert_eq!(ready.status, LinuxNumaPolicyReadinessStatus::Ready);
            assert_eq!(ready.reason, LinuxNumaPolicyReadinessReason::Ready);
            assert_eq!(ready.status.to_string(), "ready");
            assert_eq!(ready.reason.to_string(), "ready");
            assert!(ready.is_ready());

            let invalid = LinuxNumaPolicyError::InvalidNode(4096);
            let invalid_readiness = LinuxNumaPolicyReadiness::from_bind_result(Err(&invalid));
            assert_eq!(
                invalid_readiness.status,
                LinuxNumaPolicyReadinessStatus::NotReady
            );
            assert_eq!(
                invalid_readiness.reason,
                LinuxNumaPolicyReadinessReason::InvalidNode
            );
            assert_eq!(invalid_readiness.status.to_string(), "not_ready");
            assert_eq!(invalid_readiness.reason.to_string(), "invalid_node");
            assert!(!invalid_readiness.is_ready());

            let denied =
                LinuxNumaPolicyError::Syscall(io::Error::from(ErrorKind::PermissionDenied));
            assert_eq!(
                LinuxNumaPolicyReadiness::from_bind_result(Err(&denied)).reason,
                LinuxNumaPolicyReadinessReason::PermissionDenied
            );

            let other = LinuxNumaPolicyError::Syscall(io::Error::from(ErrorKind::Other));
            assert_eq!(
                LinuxNumaPolicyReadiness::from_bind_result(Err(&other)).reason,
                LinuxNumaPolicyReadinessReason::SyscallFailed
            );
        }

        #[test]
        fn parses_linux_numa_policy_readiness_lines() {
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=ready reason=ready"
                )
                .expect("ready"),
                LinuxNumaPolicyReadiness {
                    status: LinuxNumaPolicyReadinessStatus::Ready,
                    reason: LinuxNumaPolicyReadinessReason::Ready,
                }
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=not_ready reason=permission_denied"
                )
                .expect("not ready"),
                LinuxNumaPolicyReadiness {
                    status: LinuxNumaPolicyReadinessStatus::NotReady,
                    reason: LinuxNumaPolicyReadinessReason::PermissionDenied,
                }
            );
        }

        #[test]
        fn rejects_invalid_linux_numa_policy_readiness_lines() {
            assert_eq!(
                parse_linux_numa_policy_readiness_line("reason=ready").expect_err("missing status"),
                LinuxNumaPolicyReadinessLineParseError::MissingStatus
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line("memory_policy_readiness=ready")
                    .expect_err("missing reason"),
                LinuxNumaPolicyReadinessLineParseError::MissingReason
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=maybe reason=ready"
                )
                .expect_err("unknown status"),
                LinuxNumaPolicyReadinessLineParseError::UnknownStatus("maybe".to_owned())
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=ready reason=maybe"
                )
                .expect_err("unknown reason"),
                LinuxNumaPolicyReadinessLineParseError::UnknownReason("maybe".to_owned())
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=ready reason=ready extra=true"
                )
                .expect_err("extra token"),
                LinuxNumaPolicyReadinessLineParseError::InvalidToken("extra=true".to_owned())
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=ready memory_policy_readiness=not_ready reason=ready"
                )
                .expect_err("duplicate status"),
                LinuxNumaPolicyReadinessLineParseError::DuplicateStatus
            );
            assert_eq!(
                parse_linux_numa_policy_readiness_line(
                    "memory_policy_readiness=ready reason=ready reason=permission_denied"
                )
                .expect_err("duplicate reason"),
                LinuxNumaPolicyReadinessLineParseError::DuplicateReason
            );
        }
    }
}

/// Returns the operating system page size in bytes.
///
/// # Errors
///
/// Returns an error when the operating system does not report a usable page
/// size.
pub fn page_size() -> Result<usize, PageSizeError> {
    let value = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if value <= 0 {
        return Err(PageSizeError::Unavailable(io::Error::last_os_error()));
    }
    usize::try_from(value).map_err(|_| PageSizeError::InvalidValue(value))
}

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

    /// Returns the start pointer of the mapping.
    #[must_use]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
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

    /// Writes one byte per page-sized stride to fault pages into memory.
    ///
    /// # Errors
    ///
    /// Returns an error when `page_size` is zero.
    pub fn write_touch_pages(&mut self, page_size: usize) -> Result<usize, TouchPagesError> {
        if page_size == 0 {
            return Err(TouchPagesError::InvalidPageSize);
        }

        let mut touched = 0_usize;
        let mut offset = 0_usize;
        while offset < self.len {
            unsafe {
                let ptr = self.ptr.as_ptr().add(offset);
                let value = ptr.read_volatile();
                ptr.write_volatile(value.wrapping_add(1));
            }
            touched = touched.saturating_add(1);
            offset = offset.saturating_add(page_size);
        }

        Ok(touched)
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

/// Page-size lookup failures.
#[derive(Debug)]
pub enum PageSizeError {
    /// Page size was not available.
    Unavailable(io::Error),
    /// Page size did not fit in `usize`.
    InvalidValue(libc::c_long),
}

/// Page touching failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPagesError {
    /// Page size must be non-zero.
    InvalidPageSize,
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

impl fmt::Display for PageSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(source) => write!(f, "page size unavailable: {source}"),
            Self::InvalidValue(value) => write!(f, "invalid page size value: {value}"),
        }
    }
}

impl std::error::Error for PageSizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unavailable(source) => Some(source),
            Self::InvalidValue(_) => None,
        }
    }
}

impl fmt::Display for TouchPagesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPageSize => f.write_str("page size must be non-zero"),
        }
    }
}

impl std::error::Error for TouchPagesError {}

#[cfg(test)]
mod tests {
    use super::{page_size, MappedRegion, MappedRegionError, TouchPagesError};

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

    #[test]
    fn reports_page_size() {
        let size = page_size().expect("page size");

        assert!(size >= 4096);
        assert_eq!(size.count_ones(), 1);
    }

    #[test]
    fn write_touches_one_byte_per_page_stride() {
        let size = page_size().expect("page size");
        let mut region = MappedRegion::anonymous(size * 2 + 1).expect("mapped region");

        let touched = region.write_touch_pages(size).expect("touch pages");

        assert_eq!(touched, 3);
        assert_eq!(region.as_slice()[0], 1);
        assert_eq!(region.as_slice()[size], 1);
        assert_eq!(region.as_slice()[size * 2], 1);
    }

    #[test]
    fn rejects_zero_page_size_touch() {
        let mut region = MappedRegion::anonymous(4096).expect("mapped region");

        assert_eq!(
            region.write_touch_pages(0).expect_err("zero page size"),
            TouchPagesError::InvalidPageSize
        );
    }
}
