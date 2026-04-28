use std::path::Path;

use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::btrfs,
    response::{ResponseError, ResponseResult},
    string::escaped_utf8_from_bytes,
};

pub struct Du {
    pub exclusive: u64,
    pub shared: u64,
}
pub async fn du(path: impl AsRef<Path>) -> ResponseResult<Du> {
    let path = path.as_ref();

    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command
        .args(["filesystem", "du", "--summarize", "--raw"])
        .arg(path);

    execute_command_simple(command)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not get size of {path}: {e}",
                path = path.display()
            ))
        })
        .and_then(|output| {
            let output = escaped_utf8_from_bytes(output);
            let output_by_line: Vec<&str> = output.split('\n').collect();

            let usage: Vec<&str> = output_by_line
                .get(1)
                .ok_or_else(|| ResponseError::new(format!("No second line in btrfs du: {output}")))?
                .split_whitespace()
                .collect();

            let exclusive = usage
                .get(1)
                .ok_or_else(|| {
                    ResponseError::new(format!("No exclusive on second line in btrfs du: {output}"))
                })?
                .parse()
                .map_err(|e| {
                    ResponseError::new(format!(
                        "Could not convert exclusive of btrfs du {output} to u64: {e}"
                    ))
                })?;

            let shared = usage
                .get(2)
                .ok_or_else(|| {
                    ResponseError::new(format!("No shared on second line in btrfs du: {output}"))
                })?
                .parse()
                .map_err(|e| {
                    ResponseError::new(format!(
                        "Could not convert shared of btrfs du {output} to u64: {e}"
                    ))
                })?;

            Ok(Du { exclusive, shared })
        })
}

pub struct Show {
    pub total: u64,
    pub used: u64,
}
pub async fn show(path: impl AsRef<Path>) -> ResponseResult<Show> {
    let path = path.as_ref();

    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["filesystem", "show", "--raw"]).arg(path);

    execute_command_simple(command)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not get show of {path}: {e}",
                path = path.display()
            ))
        })
        .and_then(|output| {
            let output = escaped_utf8_from_bytes(output);
            let output_by_line: Vec<&str> = output.split('\n').collect();

            output_by_line
                .iter()
                .find(|line| line.trim().ends_with(&path.display().to_string()))
                .ok_or_else(|| {
                    ResponseError::new(format!(
                        "Missing {path} from show {output}",
                        path = path.display()
                    ))
                })
                .and_then(|line| {
                    let split: Vec<&str> = line.split_whitespace().collect();

                    let total = split
                        .get(4)
                        .ok_or_else(|| {
                            ResponseError::new(format!(
                                "Missing total from show {path} line {line}",
                                path = path.display()
                            ))
                        })?
                        .parse()
                        .map_err(|e| {
                            ResponseError::new(format!(
                                "Could not convert total from show {path} line {line} to u64: {e}",
                                path = path.display()
                            ))
                        })?;

                    let used = split
                        .get(6)
                        .ok_or_else(|| {
                            ResponseError::new(format!(
                                "Missing used from show {path} line {line}",
                                path = path.display()
                            ))
                        })?
                        .parse()
                        .map_err(|e| {
                            ResponseError::new(format!(
                                "Could not convert used from show {path} line {line} to u64: {e}",
                                path = path.display()
                            ))
                        })?;

                    Ok(Show { total, used })
                })
        })
}
