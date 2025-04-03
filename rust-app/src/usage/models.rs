use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sysinfo::System;

pub struct AppData {
    pub system: Mutex<System>,
}

impl Default for AppData {
    fn default() -> Self {
        AppData {
            system: Mutex::new(System::new()),
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
