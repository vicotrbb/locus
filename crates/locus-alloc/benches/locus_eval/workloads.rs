//! LOCUS-EVAL v1 workload definitions.
//!
//! This file is the versioned unit of the suite: any change to a
//! constant, arrival schedule, decode-length rule, or churn shape
//! requires bumping the suite version in
//! `documentation/evaluations/`, never editing v1 results.

pub const BLOCK_SIZE: usize = 4096;
pub const WORKERS: usize = 4;
pub const REQUESTS: usize = 64;
pub const PREFILL_BLOCKS: usize = 16;
pub const CANCEL_DECODE_STEPS: usize = 8;
// Pool sizes only apply to the locus contenders; the malloc runner
// has no pool, so they are unused in those binaries.
#[allow(dead_code)]
pub const TRACE_POOL_BLOCKS: usize = 8192;

#[allow(dead_code)]
pub const CHURN_POOL_BLOCKS: usize = 16384;
pub const CHURN_LIVE_CHUNKS: usize = 256;
pub const CHURN_CHUNK_BLOCKS: usize = 16;
pub const CHURN_STEPS: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Workload {
    SteadyDecode,
    BurstStorm,
    LongTail,
    ChurnTouch,
}

pub const TRACE_WORKLOADS: [Workload; 3] = [
    Workload::SteadyDecode,
    Workload::BurstStorm,
    Workload::LongTail,
];

impl Workload {
    pub fn name(self) -> &'static str {
        match self {
            Workload::SteadyDecode => "steady_decode",
            Workload::BurstStorm => "burst_storm",
            Workload::LongTail => "long_tail",
            Workload::ChurnTouch => "churn_touch",
        }
    }

    /// Requests admitted per owner step until all `REQUESTS` arrived.
    pub fn arrivals_per_step(self) -> usize {
        match self {
            Workload::SteadyDecode | Workload::LongTail => 4,
            Workload::BurstStorm => REQUESTS,
            Workload::ChurnTouch => 0,
        }
    }

    /// Decode length by arrival index; the request frees all blocks
    /// as one chunk when it reaches zero remaining steps.
    pub fn decode_steps(self, request_index: usize) -> usize {
        match self {
            Workload::SteadyDecode | Workload::BurstStorm => {
                if request_index % 4 == 0 {
                    CANCEL_DECODE_STEPS
                } else {
                    16 + (request_index % 3) * 16
                }
            }
            Workload::LongTail => {
                if request_index % 8 == 0 {
                    512
                } else {
                    16
                }
            }
            Workload::ChurnTouch => 0,
        }
    }

    /// Exact blocks allocated per trace (prefill plus decode).
    pub fn blocks_per_trace(self) -> usize {
        match self {
            Workload::ChurnTouch => CHURN_STEPS * CHURN_CHUNK_BLOCKS,
            _ => (0..REQUESTS)
                .map(|index| PREFILL_BLOCKS + self.decode_steps(index))
                .sum(),
        }
    }

    /// Peak live blocks if every free landed instantly, computed by
    /// replaying the trace schedule with an instant-free model. This
    /// is the denominator of the quality (overhead) ratio.
    pub fn theoretical_peak_live(self) -> usize {
        if self == Workload::ChurnTouch {
            return CHURN_LIVE_CHUNKS * CHURN_CHUNK_BLOCKS;
        }
        let mut live: Vec<(usize, usize)> = Vec::new();
        let mut arrived = 0_usize;
        let mut live_blocks = 0_usize;
        let mut peak = 0_usize;
        while arrived < REQUESTS || !live.is_empty() {
            let arriving = self.arrivals_per_step().min(REQUESTS - arrived);
            for _ in 0..arriving {
                live.push((PREFILL_BLOCKS, self.decode_steps(arrived)));
                live_blocks += PREFILL_BLOCKS;
                arrived += 1;
            }
            peak = peak.max(live_blocks);
            let mut index = 0;
            while index < live.len() {
                if live[index].1 == 0 {
                    live_blocks -= live[index].0;
                    live.swap_remove(index);
                    continue;
                }
                live[index].0 += 1;
                live[index].1 -= 1;
                live_blocks += 1;
                peak = peak.max(live_blocks);
                index += 1;
            }
        }
        peak
    }
}
