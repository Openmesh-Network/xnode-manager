use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Metadata {
    File {},
    Folder {},
    Unknown {},
}

#[derive(Serialize, Deserialize)]
pub struct Size {
    pub exclusive: u64,
    pub shared: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    pub folders: Vec<String>,
    pub files: Vec<String>,
    pub symlinks: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Entity {
    User(u32),
    Group(u32),
    Any,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct Permission {
    pub granted_to: Entity,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PathQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SourceDestinationData {
    pub source: String,
    pub destination: String,
}
