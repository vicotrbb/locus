#!/usr/bin/env bash
# Runs the full LOCUS-EVAL suite with the exact Criterion flags used for
# the recorded LOCUS-EVAL v1 results, capturing host info and raw logs.
#
# Usage: scripts/run-eval.sh [run-label]
#   run-label defaults to a timestamp; logs are written to
#   documentation/evaluations/logs/ as locus_eval_<contender>_<label>.log
#
# Two full runs per contender are the methodology minimum; rerun this
# script (with a new label) for the second run, and a third time for any
# contender whose medians disagree by more than 5 percent.

set -euo pipefail

cd "$(dirname "$0")/.."

LABEL="${1:-$(date +%Y%m%d-%H%M%S)}"
LOG_DIR="documentation/evaluations/logs"
mkdir -p "$LOG_DIR"

HOST_LOG="$LOG_DIR/locus_eval_host_${LABEL}.log"
{
    echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "uname: $(uname -a)"
    echo "rustc: $(rustc --version)"
    echo "cargo: $(cargo --version)"
    if [ "$(uname)" = "Darwin" ]; then
        sysctl -n machdep.cpu.brand_string hw.ncpu hw.memsize
    else
        grep -m1 "model name" /proc/cpuinfo || true
        nproc
        grep MemTotal /proc/meminfo || true
    fi
} > "$HOST_LOG"
echo "host info: $HOST_LOG"

# Exact flags from LOCUS-EVAL v1 (documentation/evaluations/0001-locus-eval-v1.md).
CRITERION_FLAGS=(--sample-size 20 --warm-up-time 1 --measurement-time 3)

for contender in locus jemalloc mimalloc system; do
    log="$LOG_DIR/locus_eval_${contender}_${LABEL}.log"
    echo "running locus_eval_${contender} -> $log"
    cargo bench -p locus-alloc --bench "locus_eval_${contender}" -- \
        "${CRITERION_FLAGS[@]}" 2>&1 | tee "$log"
done

echo "done. Logs are verbatim records: never edit or regenerate them."
