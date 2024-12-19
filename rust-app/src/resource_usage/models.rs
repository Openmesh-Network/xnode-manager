use serde::{Deserialize, Serialize};

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
    pub name: String,
    pub used: u64,
    pub total: u64,
}
