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

## Combined Gate

The live combined gate runs the mapped scratch validation path in one process:

```sh
cargo run -p locus-validate --example live_placement_validation_gate
```

It prints:

```text
placement_validation_gate=<status> reason=<reason>
```

Captured probe outputs can be evaluated together with:

```sh
cargo run -p locus-validate --example placement_validation_gate -- \
  memory-policy.out \
  placement-readiness.out \
  placement-proof.out
```

The example prints:

```text
placement_validation_gate=<status> reason=<reason>
```

The combined gate output parser is:

- `locus_validate::linux::parse_placement_validation_gate_output`.

The combined gate parser rejects status and reason pairs that are individually valid but incoherent together. For example, `placement_validation_gate=verified reason=memory_policy_not_ready` is malformed because a verified gate must use `reason=verified`.

Line-level parsers are also available when callers already isolated the final verdict line:

- `locus_sys::linux::parse_linux_numa_policy_readiness_line`;
- `locus_observe::parse_numa_placement_readiness_line`;
- `locus_observe::parse_numa_placement_proof_line`;
- `locus_validate::linux::parse_placement_validation_gate_line`.

## Acceptance Rules

Successful placement validation requires all of the following parsed verdicts:

- `memory_policy_readiness=ready reason=ready`;
- `placement_validation_readiness=ready reason=ready`;
- `placement_proof=verified reason=verified`;
- `placement_validation_gate=verified reason=verified`.

Any `not_ready`, `unavailable`, or `unverified` verdict is a useful validation result, but it is not proof of successful NUMA placement.

## Current Docker Verdicts

The current Docker environment still produces:

- `memory_policy_readiness=not_ready reason=permission_denied`;
- `placement_validation_readiness=not_ready reason=numa_maps_unavailable`;
- `placement_proof=unavailable reason=numa_maps_unavailable`;
- `placement_validation_gate=not_ready reason=memory_policy_not_ready`.

Those verdicts validate failure handling, parser coverage, and output stability. They do not validate page placement.
