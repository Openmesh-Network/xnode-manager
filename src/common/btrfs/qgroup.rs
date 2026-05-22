use std::path::Path;

use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::btrfs,
    response::{ResponseError, ResponseResult},
};

pub async fn limit(path: impl AsRef<Path>, size: Option<u64>) -> ResponseResult<()> {
    let path = path.as_ref();
    let size = size
        .map(|bytes| format!("{bytes}B"))
        .unwrap_or("none".to_string());
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["qgroup", "limit", "-c"]).arg(&size).arg(path);

    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not limit {path} to {size}: {e}",
                path = path.display()
            ))
        })
}
