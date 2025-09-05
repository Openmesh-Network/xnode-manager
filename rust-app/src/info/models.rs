use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlakeQuery {
    pub flake: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct FlakeMetadata {
    pub lastModified: u64,
    pub revision: String,
}

#[derive(Serialize, Deserialize)]
pub struct Flake {
    pub last_modified: u64,
    pub revision: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: u32,
    pub group: u32,
    pub description: String,
    pub home: String,
    pub login: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub id: u32,
    pub members: Vec<String>,
}
