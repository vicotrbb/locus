# Postulate 0078: Mapped Region Page Lock

Date: 2026-07-02

## Statement

`MappedRegion` should expose safe page-lock and unlock operations as the first primitive for future pinned host staging buffers.

## Rationale

GPU-near pinned staging buffers need page-locking behavior, but that behavior must stay behind the narrow `locus-sys` boundary. A safe `MappedRegion::lock_pages` and `MappedRegion::unlock_pages` pair lets allocator experiments request pinned host memory without calling `mlock` or `munlock` directly.

This is not a CUDA host registration API yet. It is the smallest Linux and POSIX memory locking primitive that can be validated under Docker and later composed with GPU locality policy.

## Experiment

Add safe `MappedRegion` methods that call `mlock` and `munlock` for the region lifetime. Return explicit errors when the operating system rejects either call.

Run host tests and Docker `locus-sys` tests. Record whether the current Docker environment permits locking a small anonymous mapping.

## Expected Result

Small mapped regions should lock and unlock successfully in normal host and Docker test environments. If an environment denies locking due to `RLIMIT_MEMLOCK` or permissions, the error should be explicit and safe.
