# Validation Output Parsers

Date: 2026-07-02

## Purpose

Locus validation probes now print stable final verdict lines. This note maps each probe to the parser that should consume its output during automated placement validation.

## Probe Output Map

| Probe | Final line | Parser |
| --- | --- | --- |
| `cargo run -p locus-sys --example mbind_region` | `memory_policy_readiness=<status> reason=<reason>` | `locus_sys::linux::parse_linux_numa_policy_readiness_output` |
| `cargo run -p locus-observe --example locality_environment` | `placement_validation_readiness=<status> reason=<reason>` | `locus_observe::parse_numa_placement_readiness_output` |
| `cargo run -p locus-alloc --example mapped_scratch_bind` | `placement_proof=<status> reason=<reason>` | `locus_observe::parse_numa_placement_proof_output` |

Line-level parsers are also available when callers already isolated the final verdict line:

- `locus_sys::linux::parse_linux_numa_policy_readiness_line`;
- `locus_observe::parse_numa_placement_readiness_line`;
- `locus_observe::parse_numa_placement_proof_line`.

## Acceptance Rules

Successful placement validation requires all of the following parsed verdicts:

- `memory_policy_readiness=ready reason=ready`;
- `placement_validation_readiness=ready reason=ready`;
- `placement_proof=verified reason=verified`.

Any `not_ready`, `unavailable`, or `unverified` verdict is a useful validation result, but it is not proof of successful NUMA placement.

## Current Docker Verdicts

The current Docker environment still produces:

- `memory_policy_readiness=not_ready reason=permission_denied`;
- `placement_validation_readiness=not_ready reason=numa_maps_unavailable`;
- `placement_proof=unavailable reason=numa_maps_unavailable`.

Those verdicts validate failure handling, parser coverage, and output stability. They do not validate page placement.
