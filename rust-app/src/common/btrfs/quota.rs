use std::path::Path;

use tokio::process::Command;

use crate::common::{command::execute_command_simple, env::btrfs, error::ResponseError};

pub async fn enable(path: impl AsRef<Path>) -> Result<(), ResponseError> {
    let path = path.as_ref();
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["quota", "enable", "--simple"]).arg(path);

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| ResponseError {
            error: format!(
                "Could not enable quotas on {path}: {e}",
                path = path.display()
            ),
        })
}
