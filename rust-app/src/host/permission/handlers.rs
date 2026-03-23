use std::path::Path;

use crate::common::{
    error::ResponseError,
    file::{read_file, write_file},
    string::escaped_utf8_from_bytes,
};

use super::models::Permission;

pub async fn get_permission(path: impl AsRef<Path>) -> Result<Permission, ResponseError> {
    let path = path.as_ref();
    let bytes = read_file(path).await?;
    serde_json::from_str(&escaped_utf8_from_bytes(bytes)).map_err(|e| {
        ResponseError::new(format!(
            "Could not convert {path} into permission: {e}",
            path = path.display()
        ))
    })
}

pub async fn set_permission(
    path: impl AsRef<Path>,
    permission: Permission,
    detect_changes: bool,
    allow_restart: bool,
) -> Result<(), ResponseError> {
    let path = path.as_ref();
    let current_permission = if detect_changes {
        get_permission(path).await?
    } else {
        Permission {
            process: None,
            disk: None,
            bind: None,
            device: None,
            extra_args: None,
        }
    };

    // TODO check changes
    // TODO apply changes
    // TODO restart if required

    let bytes = serde_json::to_string(&permission).map_err(|e| {
        ResponseError::new(format!("Could not convert {permission:?} into json: {e}"))
    })?;
    write_file(path, bytes).await
}
