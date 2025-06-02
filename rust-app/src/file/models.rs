use serde::{Deserialize, Serialize};

use crate::utils::output::Output;

#[derive(Serialize, Deserialize)]
pub struct Location {
    pub container: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadFile {
    pub location: Location,
}

#[derive(Serialize, Deserialize)]
pub struct WriteFile {
    pub location: Location,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFile {
    pub location: Location,
}

#[derive(Serialize, Deserialize)]
pub struct ReadDirectory {
    pub location: Location,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDirectory {
    pub location: Location,
    pub make_parent: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveDirectory {
    pub location: Location,
    pub make_empty: bool,
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
