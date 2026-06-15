use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
}
