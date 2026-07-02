# Postulate 0030: Live Node Numastat System Reader

Date: 2026-07-02

## Statement

Locus should provide one shared reader that builds a system-level node `numastat` snapshot from a Linux node sysfs root.

## Rationale

The live node example previously duplicated directory scanning. Placement probes will need the same operation for before-and-after node counter snapshots. Moving the scan into `locus-observe` keeps node filtering, missing-file handling, and snapshot construction consistent.

## Experiment

Add a reader that:

- scans `node*` directories under a configurable root;
- parses numeric node identifiers;
- reads each present `numastat` file;
- skips non-node entries and node directories without a `numastat` file;
- returns a `NodeNumastatSystemSnapshot`;
- updates the live example to use the shared reader.

## Expected Result

Fixture tests should cover a sysfs-like root. The Docker example should continue to report unavailable state when node `numastat` evidence is not exposed.
