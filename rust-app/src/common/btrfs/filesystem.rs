use std::path::Path;

use tokio::process::Command;

use crate::common::{command::execute_command_simple, env::btrfs, response::{ResponseError, ResponseResult}};

pub struct DuReturn {
    pub exclusive: u64,
    pub shared: u64,
}
pub async fn du(path: impl AsRef<Path>) -> ResponseResult<DuReturn> {
    let path = path.as_ref();
let mut command = Command::new(format!("{}btrfs", btrfs()));
    command
        .args(["filesystem", "du", "--summarize", "--raw"])
        .arg(path);

    execute_command_simple(command)
        .await
        .map(|output| 
            // TODO output parsing
            DuReturn {
            exclusive: 0,
            shared: 0,
        })
        .map_err(|e| ResponseError::new(format!("Could not get size of {path}: {e}", path = path.display())))
}