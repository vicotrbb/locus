//! Request affinity models for locality-aware allocation.

use crate::topology::{NodeId, Topology};

/// Stable request identifier supplied by an inference scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(pub u64);

/// GPU identifier used by scheduler and placement APIs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuId(pub String);

/// Scheduler-provided locality hints for a request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestAffinity {
    /// CPU currently expected to execute request-local work.
    pub worker_cpu: Option<usize>,
    /// GPU expected to consume request output or staging buffers.
    pub gpu: Option<GpuId>,
    /// Explicit NUMA node supplied by the scheduler when already known.
    pub preferred_node: Option<NodeId>,
}

/// Chosen home node for request-scoped allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHome {
    /// Request identifier.
    pub request_id: RequestId,
    /// Selected home NUMA node, or none when topology is unknown.
    pub node: Option<NodeId>,
    /// Short reason suitable for experiment logs and debug output.
    pub reason: &'static str,
}

/// Chooses a conservative request home node from affinity hints and topology.
///
/// This function intentionally avoids system calls. It is a deterministic
/// policy helper that allocator and scheduler layers can test directly.
#[must_use]
pub fn choose_request_home(
    request_id: RequestId,
    affinity: &RequestAffinity,
    topology: &Topology,
) -> RequestHome {
    if let Some(node) = affinity.preferred_node {
        return RequestHome {
            request_id,
            node: Some(node),
            reason: "scheduler supplied preferred NUMA node",
        };
    }

    if let Some(worker_cpu) = affinity.worker_cpu {
        if let Some(node) = topology
            .nodes
            .iter()
            .find(|node| node.cpus.contains(worker_cpu))
        {
            return RequestHome {
                request_id,
                node: Some(node.id),
                reason: "worker CPU belongs to discovered NUMA node",
            };
        }
    }

    if let Some(first_node) = topology.nodes.first() {
        return RequestHome {
            request_id,
            node: Some(first_node.id),
            reason: "falling back to first discovered NUMA node",
        };
    }

    RequestHome {
        request_id,
        node: None,
        reason: "no NUMA topology available",
    }
}

#[cfg(test)]
mod tests {
    use crate::{CpuSet, NodeId, NumaNode, RequestAffinity, RequestId, Topology};

    use super::choose_request_home;

    fn topology() -> Topology {
        Topology {
            nodes: vec![
                NumaNode {
                    id: NodeId(0),
                    cpus: CpuSet::from_cpus(vec![0, 1, 2, 3]),
                    meminfo: None,
                },
                NumaNode {
                    id: NodeId(1),
                    cpus: CpuSet::from_cpus(vec![4, 5, 6, 7]),
                    meminfo: None,
                },
            ],
            pci_devices: Vec::new(),
        }
    }

    #[test]
    fn honors_scheduler_preferred_node() {
        let home = choose_request_home(
            RequestId(42),
            &RequestAffinity {
                preferred_node: Some(NodeId(1)),
                worker_cpu: Some(0),
                gpu: None,
            },
            &topology(),
        );

        assert_eq!(home.node, Some(NodeId(1)));
        assert_eq!(home.request_id, RequestId(42));
    }

    #[test]
    fn chooses_node_from_worker_cpu() {
        let home = choose_request_home(
            RequestId(7),
            &RequestAffinity {
                worker_cpu: Some(5),
                ..RequestAffinity::default()
            },
            &topology(),
        );

        assert_eq!(home.node, Some(NodeId(1)));
    }

    #[test]
    fn falls_back_to_first_discovered_node() {
        let home = choose_request_home(RequestId(1), &RequestAffinity::default(), &topology());

        assert_eq!(home.node, Some(NodeId(0)));
    }

    #[test]
    fn returns_none_when_topology_is_empty() {
        let home = choose_request_home(
            RequestId(1),
            &RequestAffinity::default(),
            &Topology::default(),
        );

        assert_eq!(home.node, None);
    }
}
