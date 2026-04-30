use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub total: u64,
    pub free: u64,
    pub available: u64,
}
