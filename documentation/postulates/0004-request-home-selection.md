# Postulate 0004: Request Home Selection

Date: 2026-07-02

## Statement

Request-scoped allocation needs a deterministic home-node selection model before Locus builds request-affine arenas.

## Rationale

The research notes emphasize that inference memory placement depends on scheduler affinity. A request should carry CPU, GPU, and preferred-node hints so allocation policy can stay explicit instead of relying on accidental first-touch behavior.

## Experiment

Add safe `locus-core` request affinity types and a conservative `choose_request_home` helper that:

- honors a scheduler-supplied preferred NUMA node;
- derives a node from the worker CPU and discovered topology;
- falls back to the first discovered NUMA node;
- returns no home node when topology is unavailable.

## Expected Result

The helper should be deterministic, testable, and free of system calls. It should provide the policy input needed by a later request-scoped scratch arena manager.
