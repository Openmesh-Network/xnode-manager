use std::path::Path;

use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::btrfs,
    response::{ResponseError, ResponseResult},
};

pub async fn enable(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["quota", "enable", "--simple"]).arg(path);

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not enable quotas on {path}: {e}",
                path = path.display()
            ))
        })
}
