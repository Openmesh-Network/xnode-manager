use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub total: u64,
    pub used: u64,
}
