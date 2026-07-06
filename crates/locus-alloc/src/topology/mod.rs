//! NUMA topology types and Linux sysfs discovery.
//!
//! The data types are always available; live discovery from `/sys` lives
//! in [`discovery`] behind the `numa` feature and is Linux-only
//! (experiments 0001-0045 establish the sysfs evidence readers this
//! design relies on).

use crate::cpuset::CpuSet;

/// Linux NUMA node identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

/// Discovered NUMA node information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaNode {
    /// NUMA node identifier.
    pub id: NodeId,
    /// CPUs reported as local to this node.
    pub cpus: CpuSet,
    /// Raw sysfs meminfo content when available.
    pub meminfo: Option<String>,
}

/// Discovered PCI locality information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PciDevice {
    /// PCI bus-device-function identifier.
    pub bdf: String,
    /// NUMA node reported by sysfs, or none when the kernel reports unknown.
    pub numa_node: Option<NodeId>,
    /// CPUs reported as local to this PCI device when available.
    pub local_cpus: Option<CpuSet>,
}

/// Host topology relevant to Locus placement decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Topology {
    /// NUMA nodes sorted by node identifier.
    pub nodes: Vec<NumaNode>,
    /// PCI devices sorted by BDF.
    pub pci_devices: Vec<PciDevice>,
}

impl Topology {
    /// Returns true when no NUMA nodes were discovered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Looks up a NUMA node by identifier.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&NumaNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Looks up a PCI device by BDF.
    #[must_use]
    pub fn pci_device(&self, bdf: &str) -> Option<&PciDevice> {
        self.pci_devices.iter().find(|device| device.bdf == bdf)
    }
}

#[cfg(all(feature = "numa", target_os = "linux"))]
pub mod discovery;
