use std::path::Path;

use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::btrfs,
    response::{ResponseError, ResponseResult},
};

pub async fn create(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command
        .args(["--quiet", "subvolume", "create", "--parents"])
        .arg(path);

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not create subvolume {path}: {e}",
                path = path.display()
            ))
        })
}

pub async fn delete(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command
        .args(["--quiet", "subvolume", "delete", "--recursive"])
        .arg(path);

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not delete subvolume {path}: {e}",
                path = path.display()
            ))
        })
}

/// Use readonly: true to take a snapshot, use readonly: false to restore a snapshot
pub async fn snapshot(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    readonly: bool,
) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command
        .args(["--quiet", "subvolume", "snapshot"])
        .args([source, destination]);
    if readonly {
        command.arg("-r");
    }

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not snapshot {source} to {destination}: {e}",
                source = source.display(),
                destination = destination.display()
            ))
        })
}
