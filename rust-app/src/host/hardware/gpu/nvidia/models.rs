use nvml_wrapper::{Nvml, error::NvmlError};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub struct AppData {
    pub nvml: Mutex<Result<Nvml, NvmlError>>,
}

impl Default for AppData {
    fn default() -> Self {
        AppData {
            nvml: Mutex::new(Nvml::init()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Gpu {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total: u64,
    pub available: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    /// In percent
    pub compute: u32,
    pub memory: MemoryUsage,
    /// In milliwatts
    pub power: Option<u32>,
}
