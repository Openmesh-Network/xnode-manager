use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
}
