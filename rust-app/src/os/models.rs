use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OSConfiguration {
    pub flake: String,
    pub owner: String,
}
