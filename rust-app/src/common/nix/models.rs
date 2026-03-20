use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct CliFlakeMetadata {
    pub lastModified: u64,
    pub revision: String,
}

#[derive(Serialize, Deserialize)]
pub struct FlakeMetadata {
    pub last_modified: u64,
    pub revision: String,
}
