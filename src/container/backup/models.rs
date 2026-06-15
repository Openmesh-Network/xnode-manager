use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub receive: String,
    pub common: Option<Vec<String>>,
}
