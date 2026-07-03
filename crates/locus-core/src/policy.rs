//! Placement policy models for domain allocators.

use std::collections::BTreeSet;

use crate::topology::{NodeId, Topology};

/// Memory class used to separate placement and lifetime behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryClass {
    /// Request metadata and control state.
    RequestMeta,
    /// Private KV cache blocks for one request or sequence.
    KvPrivate,
    /// Shared, read-mostly KV prefix data.
    KvSharedPrefix,
    /// Short-lived scratch buffers.
    Scratch,
    /// Read-mostly large regions, such as mapped weights or lookup tables.
    ReadMostlyLarge,
    /// GPU staging buffers in page-locked host memory.
    PinnedHost,
}

/// Expected allocation lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeHint {
    /// Allocation is reset at an operator, iteration, or decode-step boundary.
    Step,
    /// Allocation is tied to a request or sequence.
    Request,
    /// Allocation is shared across requests.
    Shared,
    /// Allocation is process-lived.
    Process,
}

/// Sorted set of NUMA node identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeSet {
    nodes: Vec<NodeId>,
}

impl NodeSet {
    /// Builds a sorted, deduplicated node set.
    #[must_use]
    pub fn from_nodes(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        let set = nodes.into_iter().collect::<BTreeSet<_>>();
        Self {
            nodes: set.into_iter().collect(),
        }
    }

    /// Builds a single-node set.
    #[must_use]
    pub fn one(node: NodeId) -> Self {
        Self { nodes: vec![node] }
    }

    /// Returns true when the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates over nodes in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().copied()
    }

    /// Returns the nodes as a sorted vector.
    #[must_use]
    pub fn to_vec(&self) -> Vec<NodeId> {
        self.nodes.clone()
    }
}

/// Abstract placement policy. OS-specific code is responsible for applying it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementPolicy {
    /// Let Linux allocate from the node local to the faulting CPU.
    Local,
    /// Bind allocation to the provided node set.
    Bind(NodeSet),
    /// Prefer one NUMA node and allow fallback.
    Prefer(NodeId),
    /// Prefer a set of NUMA nodes and allow fallback.
    PreferMany(NodeSet),
    /// Interleave pages across a node set.
    Interleave(NodeSet),
    /// Resolve the nearest NUMA node to a GPU, then lower to another policy.
    NearGpu(String),
}

/// Input for an initial locality decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementRequest {
    /// Memory class.
    pub memory_class: MemoryClass,
    /// Lifetime hint.
    pub lifetime: LifetimeHint,
    /// Preferred NUMA node from scheduler or request affinity.
    pub preferred_node: Option<NodeId>,
    /// Preferred GPU identifier for near-GPU placement.
    pub preferred_gpu: Option<String>,
}

/// Locality decision returned by the policy layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalityDecision {
    /// Selected policy.
    pub policy: PlacementPolicy,
    /// Short reason suitable for logs and experiment notes.
    pub reason: &'static str,
}

/// Chooses a conservative initial policy for a memory class.
#[must_use]
pub fn choose_initial_policy(request: &PlacementRequest) -> LocalityDecision {
    if matches!(request.memory_class, MemoryClass::PinnedHost) {
        if let Some(gpu) = &request.preferred_gpu {
            return LocalityDecision {
                policy: PlacementPolicy::NearGpu(gpu.clone()),
                reason: "pinned host buffers should follow GPU locality",
            };
        }
    }

    if let Some(node) = request.preferred_node {
        let policy = match request.memory_class {
            MemoryClass::RequestMeta
            | MemoryClass::KvPrivate
            | MemoryClass::Scratch
            | MemoryClass::PinnedHost => PlacementPolicy::Bind(NodeSet::one(node)),
            MemoryClass::KvSharedPrefix | MemoryClass::ReadMostlyLarge => {
                PlacementPolicy::Prefer(node)
            }
        };

        return LocalityDecision {
            policy,
            reason: "preferred node supplied by scheduler or caller",
        };
    }

    LocalityDecision {
        policy: PlacementPolicy::Local,
        reason: "no affinity supplied, using local first-touch behavior",
    }
}

/// Resolves topology-dependent placement policies to concrete host policies.
///
/// `NearGpu` is a logical policy until topology discovery can map the GPU BDF
/// to a NUMA node. This helper performs that deterministic lowering without
/// calling the operating system or applying memory policy.
#[must_use]
pub fn resolve_topology_policy(
    decision: &LocalityDecision,
    topology: &Topology,
) -> LocalityDecision {
    let PlacementPolicy::NearGpu(gpu) = &decision.policy else {
        return decision.clone();
    };

    let Some(device) = topology.pci_device(gpu) else {
        return LocalityDecision {
            policy: PlacementPolicy::Local,
            reason: "GPU PCI device was not discovered, using local first-touch behavior",
        };
    };

    let Some(node) = device.numa_node else {
        return LocalityDecision {
            policy: PlacementPolicy::Local,
            reason: "GPU PCI device has no reported NUMA node, using local first-touch behavior",
        };
    };

    LocalityDecision {
        policy: PlacementPolicy::Bind(NodeSet::one(node)),
        reason: "GPU PCI device reports a NUMA node",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        choose_initial_policy, resolve_topology_policy, LifetimeHint, LocalityDecision,
        MemoryClass, NodeSet, PlacementPolicy, PlacementRequest,
    };
    use crate::topology::{NodeId, PciDevice, Topology};

    #[test]
    fn binds_request_private_memory_to_preferred_node() {
        let request = PlacementRequest {
            memory_class: MemoryClass::KvPrivate,
            lifetime: LifetimeHint::Request,
            preferred_node: Some(NodeId(1)),
            preferred_gpu: None,
        };

        let decision = choose_initial_policy(&request);

        assert_eq!(
            decision.policy,
            PlacementPolicy::Bind(NodeSet::one(NodeId(1)))
        );
    }

    #[test]
    fn picks_near_gpu_for_pinned_host_memory() {
        let request = PlacementRequest {
            memory_class: MemoryClass::PinnedHost,
            lifetime: LifetimeHint::Process,
            preferred_node: Some(NodeId(0)),
            preferred_gpu: Some("0000:81:00.0".to_owned()),
        };

        let decision = choose_initial_policy(&request);

        assert_eq!(
            decision.policy,
            PlacementPolicy::NearGpu("0000:81:00.0".to_owned())
        );
    }

    #[test]
    fn resolves_near_gpu_to_discovered_pci_numa_node() {
        let decision = LocalityDecision {
            policy: PlacementPolicy::NearGpu("0000:81:00.0".to_owned()),
            reason: "logical near GPU policy",
        };
        let topology = Topology {
            pci_devices: vec![PciDevice {
                bdf: "0000:81:00.0".to_owned(),
                numa_node: Some(NodeId(1)),
                local_cpus: None,
            }],
            ..Topology::default()
        };

        let resolved = resolve_topology_policy(&decision, &topology);

        assert_eq!(
            resolved.policy,
            PlacementPolicy::Bind(NodeSet::one(NodeId(1)))
        );
        assert_eq!(resolved.reason, "GPU PCI device reports a NUMA node");
    }

    #[test]
    fn resolves_missing_gpu_to_local_policy() {
        let decision = LocalityDecision {
            policy: PlacementPolicy::NearGpu("0000:81:00.0".to_owned()),
            reason: "logical near GPU policy",
        };

        let resolved = resolve_topology_policy(&decision, &Topology::default());

        assert_eq!(resolved.policy, PlacementPolicy::Local);
        assert_eq!(
            resolved.reason,
            "GPU PCI device was not discovered, using local first-touch behavior"
        );
    }

    #[test]
    fn resolves_gpu_without_numa_node_to_local_policy() {
        let decision = LocalityDecision {
            policy: PlacementPolicy::NearGpu("0000:81:00.0".to_owned()),
            reason: "logical near GPU policy",
        };
        let topology = Topology {
            pci_devices: vec![PciDevice {
                bdf: "0000:81:00.0".to_owned(),
                numa_node: None,
                local_cpus: None,
            }],
            ..Topology::default()
        };

        let resolved = resolve_topology_policy(&decision, &topology);

        assert_eq!(resolved.policy, PlacementPolicy::Local);
        assert_eq!(
            resolved.reason,
            "GPU PCI device has no reported NUMA node, using local first-touch behavior"
        );
    }

    #[test]
    fn preserves_non_topology_dependent_policy() {
        let decision = LocalityDecision {
            policy: PlacementPolicy::Bind(NodeSet::one(NodeId(0))),
            reason: "caller supplied concrete policy",
        };

        let resolved = resolve_topology_policy(&decision, &Topology::default());

        assert_eq!(resolved, decision);
    }
}
