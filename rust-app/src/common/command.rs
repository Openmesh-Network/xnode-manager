use std::{fmt::Display, io::Error};

use tokio::process::Command;

use crate::common::{env::systemd, string::escaped_utf8_from_bytes};

pub enum SimpleCommandError {
    OutputError { output: Vec<u8> },
    CommandError { e: Error },
}
impl Display for SimpleCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                SimpleCommandError::OutputError { output } => {
                    escaped_utf8_from_bytes(&output)
                }
                SimpleCommandError::CommandError { e } => e.to_string(),
            }
        )
    }
}

pub type SimpleCommandResult = Result<Vec<u8>, SimpleCommandError>;
pub async fn execute_command_simple(mut command: Command) -> SimpleCommandResult {
    log::info!("Executing command (simple): {:?}", command);

    match command.output().await {
        Ok(output_raw) => {
            if !output_raw.status.success() {
                return Err(SimpleCommandError::OutputError {
                    output: output_raw.stderr,
                });
            }

            Ok(output_raw.stdout)
        }
        Err(e) => Err(SimpleCommandError::CommandError { e }),
    }
}

pub struct CommandStream {
    scope: String,
    name: String,
    request_id: String,
    process: String,
}
pub type StreamCommandResult = Result<(), ()>;
pub async fn execute_command_stream(
    command: Command,
    stream: &CommandStream,
    action: &str,
) -> StreamCommandResult {
    log::info!("Executing command (stream): {:?}", command);

    let CommandStream {
        scope,
        name,
        request_id,
        process,
    } = stream;
    let base_command = command.into_std();

    let mut command = Command::new(format!("{}systemd-run", systemd()));
    command
        .args([
            "--wait",
            "--quiet",
            "--collect",
            "--unit",
            &format!("{scope}-{name}-command-{request_id}-{process}-{action}.service"),
            "--slice",
            &format!("{action}-{process}-command-{name}-{scope}.slice"),
        ])
        .arg(base_command.get_program())
        .args(base_command.get_args());

    match command.output().await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
