use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateData {
    pub inputs: Vec<String>,
}
