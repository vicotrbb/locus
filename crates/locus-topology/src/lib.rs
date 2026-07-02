//! Linux sysfs topology discovery.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use locus_core::{CpuSet, NodeId, NumaNode, PciDevice, Topology};

/// Discovery configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    /// Root of the sysfs tree. Production code should use `/sys`.
    pub sysfs_root: PathBuf,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            sysfs_root: PathBuf::from("/sys"),
        }
    }
}

/// Discovers topology from `/sys`.
///
/// # Errors
///
/// Returns any filesystem or parsing error encountered while reading sysfs.
pub fn discover() -> io::Result<Topology> {
    discover_with_config(&DiscoveryConfig::default())
}

/// Discovers topology using the provided configuration.
///
/// # Errors
///
/// Returns any filesystem or parsing error encountered while reading sysfs.
pub fn discover_with_config(config: &DiscoveryConfig) -> io::Result<Topology> {
    discover_from_sysfs(&config.sysfs_root)
}

/// Discovers topology from a sysfs root.
///
/// # Errors
///
/// Returns any filesystem or parsing error encountered while reading the
/// supplied sysfs tree.
pub fn discover_from_sysfs(sysfs_root: impl AsRef<Path>) -> io::Result<Topology> {
    let sysfs_root = sysfs_root.as_ref();
    let mut topology = Topology {
        nodes: discover_nodes(sysfs_root)?,
        pci_devices: discover_pci_devices(sysfs_root)?,
    };
    topology.nodes.sort_by_key(|node| node.id);
    topology
        .pci_devices
        .sort_by(|left, right| left.bdf.cmp(&right.bdf));
    Ok(topology)
}

fn discover_nodes(sysfs_root: &Path) -> io::Result<Vec<NumaNode>> {
    let node_root = sysfs_root.join("devices/system/node");
    if !node_root.exists() {
        return Ok(Vec::new());
    }

    let mut nodes = Vec::new();
    for entry in fs::read_dir(node_root)? {
        let entry = entry?;
        let Some(node_id) = parse_node_dir_name(&entry.file_name()) else {
            continue;
        };

        let cpulist = read_trimmed(entry.path().join("cpulist")).unwrap_or_default();
        let cpus = CpuSet::parse_linux_list(&cpulist).map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid cpulist for node{}: {source}", node_id.0),
            )
        })?;
        let meminfo = read_trimmed(entry.path().join("meminfo")).ok();

        nodes.push(NumaNode {
            id: node_id,
            cpus,
            meminfo,
        });
    }

    Ok(nodes)
}

fn discover_pci_devices(sysfs_root: &Path) -> io::Result<Vec<PciDevice>> {
    let pci_root = sysfs_root.join("bus/pci/devices");
    if !pci_root.exists() {
        return Ok(Vec::new());
    }

    let mut devices = Vec::new();
    for entry in fs::read_dir(pci_root)? {
        let entry = entry?;
        let bdf = entry.file_name().to_string_lossy().into_owned();
        let device_path = entry.path();

        let numa_node = read_trimmed(device_path.join("numa_node"))
            .ok()
            .and_then(|value| parse_pci_numa_node(&value).ok())
            .flatten();

        let local_cpus = read_trimmed(device_path.join("local_cpulist"))
            .ok()
            .and_then(|value| CpuSet::parse_linux_list(&value).ok());

        devices.push(PciDevice {
            bdf,
            numa_node,
            local_cpus,
        });
    }

    Ok(devices)
}

fn read_trimmed(path: impl AsRef<Path>) -> io::Result<String> {
    Ok(fs::read_to_string(path)?.trim().to_owned())
}

fn parse_node_dir_name(name: &OsStr) -> Option<NodeId> {
    name.to_str()?
        .strip_prefix("node")?
        .parse::<u32>()
        .ok()
        .map(NodeId)
}

fn parse_pci_numa_node(value: &str) -> io::Result<Option<NodeId>> {
    let parsed = value.trim().parse::<i32>().map_err(|source| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid PCI numa_node value: {source}"),
        )
    })?;

    if parsed < 0 {
        Ok(None)
    } else {
        Ok(Some(NodeId(u32::try_from(parsed).map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PCI numa_node does not fit in u32: {source}"),
            )
        })?)))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use locus_core::NodeId;
    use tempfile::TempDir;

    use super::discover_from_sysfs;

    #[test]
    fn discovers_numa_nodes_and_pci_locality_from_sysfs_fixture() {
        let temp = TempDir::new().expect("tempdir");
        let sysfs = temp.path();

        fs::create_dir_all(sysfs.join("devices/system/node/node0")).expect("node0 dir");
        fs::create_dir_all(sysfs.join("devices/system/node/node1")).expect("node1 dir");
        fs::write(sysfs.join("devices/system/node/node0/cpulist"), "0-3\n").expect("node0 cpus");
        fs::write(sysfs.join("devices/system/node/node1/cpulist"), "4-7\n").expect("node1 cpus");
        fs::write(
            sysfs.join("devices/system/node/node1/meminfo"),
            "Node 1 MemTotal: 1024 kB\n",
        )
        .expect("node1 meminfo");

        let pci = sysfs.join("bus/pci/devices/0000:81:00.0");
        fs::create_dir_all(&pci).expect("pci dir");
        fs::write(pci.join("numa_node"), "1\n").expect("pci numa node");
        fs::write(pci.join("local_cpulist"), "4-5\n").expect("pci local cpus");

        let topology = discover_from_sysfs(sysfs).expect("discover topology");

        assert_eq!(topology.nodes.len(), 2);
        assert_eq!(topology.nodes[0].id, NodeId(0));
        assert_eq!(topology.nodes[1].cpus.to_vec(), vec![4, 5, 6, 7]);

        let device = topology
            .pci_device("0000:81:00.0")
            .expect("pci device should be discovered");
        assert_eq!(device.numa_node, Some(NodeId(1)));
        assert_eq!(
            device.local_cpus.as_ref().expect("local cpus").to_vec(),
            vec![4, 5]
        );
    }

    #[test]
    fn treats_negative_pci_numa_node_as_unknown() {
        let temp = TempDir::new().expect("tempdir");
        let sysfs = temp.path();

        let pci = sysfs.join("bus/pci/devices/0000:00:1f.0");
        fs::create_dir_all(&pci).expect("pci dir");
        fs::write(pci.join("numa_node"), "-1\n").expect("pci numa node");

        let topology = discover_from_sysfs(sysfs).expect("discover topology");
        let device = topology
            .pci_device("0000:00:1f.0")
            .expect("pci device should be discovered");

        assert_eq!(device.numa_node, None);
    }
}
