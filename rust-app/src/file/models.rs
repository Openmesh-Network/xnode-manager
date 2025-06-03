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
