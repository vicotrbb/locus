//! Core data types for Locus locality decisions.

pub mod cpuset;
pub mod policy;
pub mod request;
pub mod topology;

pub use cpuset::{CpuSet, CpuSetParseError};
pub use policy::{
    choose_initial_policy, resolve_topology_policy, LifetimeHint, LocalityDecision, MemoryClass,
    NodeSet, PlacementPolicy, PlacementRequest,
};
pub use request::{choose_request_home, GpuId, RequestAffinity, RequestHome, RequestId};
pub use topology::{NodeId, NumaNode, PciDevice, Topology};
