//! Core data types for Locus locality decisions.

pub mod cpuset;
pub mod policy;
pub mod topology;

pub use cpuset::{CpuSet, CpuSetParseError};
pub use policy::{
    choose_initial_policy, LifetimeHint, LocalityDecision, MemoryClass, NodeSet, PlacementPolicy,
    PlacementRequest,
};
pub use topology::{NodeId, NumaNode, PciDevice, Topology};
