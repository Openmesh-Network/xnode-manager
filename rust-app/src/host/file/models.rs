use serde::{Deserialize, Serialize};

use crate::common::file::Permission;

#[derive(Serialize, Deserialize)]
pub struct MetadataQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SizeQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadFileQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct MoveData {
    pub source: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct WriteFileQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFileQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct CopyFileData {
    pub source: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadFolderQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateFolderQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFolderQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct CopyFolderData {
    pub source: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPermissionsQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetPermissionsQuery {
    pub path: String,
}

pub type SetPermissionsData = Vec<Permission>;
