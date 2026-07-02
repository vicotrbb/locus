# Postulate 0022: Bind Probe Numa Maps Correlation

Date: 2026-07-02

## Statement

The mapped scratch bind probe should attempt to correlate its own mapping start address with live `/proc/self/numa_maps` evidence after page touch.

## Rationale

Printing the mapping identity is useful, but a validation probe should also perform the obvious local lookup when `numa_maps` is readable. That makes successful future runs easier to interpret and preserves graceful behavior in containers where `numa_maps` is unavailable.

## Experiment

Update the mapped scratch bind example to:

- capture the arena mapping start address;
- apply the NUMA policy attempt;
- write-touch the arena pages;
- read `/proc/self/numa_maps`;
- print a matching policy and per-node pages when the mapping start is found;
- print explicit unavailable or missing status when evidence cannot be read or matched.

## Expected Result

The example should compile under all-target checks. In the current Docker environment, `mbind` should still return `EPERM` and `numa_maps` should remain unavailable.
