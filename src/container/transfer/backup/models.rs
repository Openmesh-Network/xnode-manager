use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub send: Vec<String>,
    pub receive: String,
    pub common: Option<Vec<String>>,
}
