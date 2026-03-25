use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum CPUWeight {
    Idle,
    Value(u64),
}

impl Display for CPUWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CPUWeight::Idle => "idle".to_string(),
                CPUWeight::Value(v) => v.to_string(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CPUPermission {
    /// In case there is more work than compute power, in what relative priority to allocate compute to this process. (default 100)
    pub weight: Option<CPUWeight>,
    /// Maximum compute power this process is allowed to use. (e.g. 100 is one core, 250 is two and a half cores)
    pub max: Option<u64>,
    /// Specific core indexes, the process will only run on these cores.
    pub allowed_cores: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MemoryPermission {
    /// Hard limit on memory this process is allowed to use in bytes.
    pub max: Option<u64>,
    /// Memory usage may go above the limit if unavoidable, but the processes are heavily slowed down and memory is taken away aggressively in such cases.
    pub soft_max: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SubprocessPermission {
    /// Maximum number of subprocesses this process is allowed to spawn.
    pub max: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BandwidthPermission {
    pub read: Option<u64>,
    pub write: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IopsPermission {
    pub read: Option<u64>,
    pub write: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InputOutputPermission {
    /// In case there is more work than IO, in what relative priority to allocate IO to this process. (default 100)
    pub weight: Option<CPUWeight>,
    /// Maximum block IO bandwidth this process is allowed to use in bytes.
    pub max_bandwidth: Option<BandwidthPermission>,
    /// Maximum block IO IOs-per-Second this process is allowed to use.
    pub max_iops: Option<IopsPermission>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProcessPermission {
    pub cpu: Option<CPUPermission>,
    pub memory: Option<MemoryPermission>,
    pub subprocess: Option<SubprocessPermission>,
    pub io: Option<HashMap<String, InputOutputPermission>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommandProcessPermission {
    pub total: Option<ProcessPermission>,
    pub build: Option<ProcessPermission>,
    pub update: Option<ProcessPermission>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScopedProcessPermission {
    pub total: Option<ProcessPermission>,
    pub run: Option<ProcessPermission>,
    pub command: Option<CommandProcessPermission>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DiskPermission {
    pub total: Option<u64>,
    pub data: Option<u64>,
    pub backup: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BindPermission {
    pub path: String,
    pub readonly: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DevicePermission {
    pub read: bool,
    pub write: bool,
    pub mknod: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Permission {
    pub process: Option<ScopedProcessPermission>,
    pub disk: Option<DiskPermission>,
    pub bind: Option<HashMap<String, BindPermission>>,
    pub device: Option<HashMap<String, DevicePermission>>,
    pub extra_args: Option<Vec<String>>,
}
