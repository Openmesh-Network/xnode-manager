use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct DiskOptions {
    pub usage: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub id: String,
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub total: u64,
    pub used: u64,
    pub read: u64,
    pub written: u64,
}
