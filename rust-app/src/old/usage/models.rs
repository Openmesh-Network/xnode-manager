use std::sync::Mutex;

use nvml_wrapper::{Nvml, error::NvmlError};
use serde::{Deserialize, Serialize};
use sysinfo::System;

pub struct AppData {
    pub system: Mutex<System>,
    pub nvml: Result<Mutex<Nvml>, NvmlError>,
}

impl Default for AppData {
    fn default() -> Self {
        AppData {
            system: Mutex::new(System::new()),
            nvml: Nvml::init().map(Mutex::new),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CpuUsage {
    pub name: String,
    pub used: f32,
    pub frequency: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryUsage {
    pub used: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DiskUsage {
    pub mount_point: String,
    pub used: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkUsage {
    pub name: String,
    pub mac: String,
    pub addresses: Vec<String>,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Serialize, Deserialize)]
pub enum GpuUsage {
    Nvidia {
        id: String,
        name: String,
        compute: f32,
        memory: MemoryUsage,
        power: Option<u32>,
    },
}
