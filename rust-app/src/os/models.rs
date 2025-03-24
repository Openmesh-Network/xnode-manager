use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OSChange {
    pub flake: Option<String>,
    pub update_inputs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OSConfiguration {
    pub flake: String,
    pub flake_lock: String,
}
