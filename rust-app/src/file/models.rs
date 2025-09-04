use serde::{Deserialize, Serialize};

use crate::utils::output::Output;

#[derive(Serialize, Deserialize)]
pub struct ReadFile {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct WriteFile {
    pub path: String,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFile {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadDirectory {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDirectory {
    pub path: String,
    pub make_parent: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveDirectory {
    pub path: String,
    pub make_empty: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetPermissions {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetPermissions {
    pub path: String,
    pub permissions: Vec<Permission>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub content: Output,
}

#[derive(Serialize, Deserialize)]
pub struct Directory {
    pub directories: Vec<String>,
    pub files: Vec<String>,
    pub symlinks: Vec<String>,
    pub unknown: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Entity {
    User(String),
    Group(String),
    Any,
    Max,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct Permission {
    pub granted_to: Entity,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub default: bool,
}
