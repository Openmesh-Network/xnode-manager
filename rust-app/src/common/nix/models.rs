use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateData {
    pub inputs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum ApplyWhen {
    Now,
    NextBoot,
}

#[derive(Serialize, Deserialize)]
pub struct ApplyQuery {
    pub when: Option<ApplyWhen>,
}

#[derive(Serialize, Deserialize)]
pub struct FlakeQuery {
    pub flake: String,
}

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

#[derive(Serialize, Deserialize)]
pub struct EvalQuery {
    pub statement: String,
    /// statement is a suffix for /flake#nixosConfiguration.xnode.
    pub config: Option<bool>,
}
