use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CPUWeight {
    Idle,
    Value(u64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CPUPermission {
    /// In case there is more work than compute power, in what relative priority to allocate compute to this process. (default 100)
    pub weight: Option<CPUWeight>,
    /// Maximum compute power this process is allowed to use. (e.g. 100 is one core, 250 is two and a half cores)
    pub max: Option<f64>,
    /// Specific core indexes, the process will only run on these cores.
    pub allowed_cores: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryPermission {
    /// Hard limit on memory this process is allowed to use in bytes.
    pub max: Option<u64>,
    /// Memory usage may go above the limit if unavoidable, but the processes are heavily slowed down and memory is taken away aggressively in such cases.
    pub soft_max: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubprocessPermission {
    /// Maximum number of subprocesses this process is allowed to spawn.
    pub max: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputOutputPermission {
    /// In case there is more work than IO, in what relative priority to allocate IO to this process. (default 100)
    pub weight: Option<CPUWeight>,
    /// Maximum block IO bandwidth this process is allowed to use in bytes.
    pub ax_bandwidth: Option<f64>,
    /// Maximum block IO IOs-per-Second this process is allowed to use.
    pub max_iops: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessPermission {
    pub cpu: CPUPermission,
    pub memory: MemoryPermission,
    pub subprocess: SubprocessPermission,
    pub io: InputOutputPermission,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandProcessPermission {
    pub total: ProcessPermission,
    pub build: ProcessPermission,
    pub update: ProcessPermission,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScopedProcessPermission {
    pub total: ProcessPermission,
    pub run: ProcessPermission,
    pub command: CommandProcessPermission,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskPermission {
    pub total: Option<u64>,
    pub data: Option<u64>,
    pub backup: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevicePermission {
    pub read: bool,
    pub write: bool,
    pub mknod: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Permission {
    pub process: Option<ScopedProcessPermission>,
    pub disk: Option<DiskPermission>,
    pub bind: Option<HashMap<String, String>>,
    pub device: Option<HashMap<String, DevicePermission>>,
    pub extra_args: Option<Vec<String>>,
}
