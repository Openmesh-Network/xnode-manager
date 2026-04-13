use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Metadata {
    File {},
    Folder {},
    Link {},
    Unknown {},
}

#[derive(Serialize, Deserialize)]
pub struct Size {
    pub exclusive: u64,
    pub shared: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ReadFolderOptions {
    pub metadata: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct FolderItem {
    pub name: String,
    pub metadata: Option<Metadata>,
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
