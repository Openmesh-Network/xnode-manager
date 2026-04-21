use std::{ffi::OsStr, fmt::Display, io::Error};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::common::{env::systemd, string::escaped_utf8_from_bytes};

#[derive(Serialize, Deserialize)]
pub struct ResponseCommand {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub enum CommandAfterCondition {
    Always,
    Success,
}

#[derive(Serialize, Deserialize)]
pub enum CommandAfter {
    Command {
        id: String,
        condition: Option<CommandAfterCondition>,
    },
    Date {
        /// Epoch time in seconds
        date: u64,
    },
}

#[derive(Serialize, Deserialize)]
pub struct CommandOptions {
    pub after: Option<CommandAfter>,
}

impl AsRef<CommandOptions> for CommandOptions {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub enum SimpleCommandError {
    OutputError { output: Vec<u8> },
    CommandError { e: Error },
}
impl Display for SimpleCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SimpleCommandError::OutputError { output } => {
                    escaped_utf8_from_bytes(output)
                }
                SimpleCommandError::CommandError { e } => e.to_string(),
            }
        )
    }
}

pub type SimpleCommandResult = Result<Vec<u8>, SimpleCommandError>;
pub async fn execute_command_simple(mut command: Command) -> SimpleCommandResult {
    log::info!("Executing command: {:?}", command);

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

pub async fn execute_command_wrapped(
    command: Command,
    name: &str,
    machine: Option<impl AsRef<str>>,
    options: impl AsRef<CommandOptions>,
) -> SimpleCommandResult {
    let mut base_command = command.into_std();
    let options = options.as_ref();

    let mut command = Command::new(format!("{}systemd-run", systemd()));
    command.args([
        "--wait",
        "--quiet",
        "--collect",
        "--property",
        "Type=oneshot",
        "--unit",
        &get_wrapped_unit(name),
    ]);

    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    if let Some(after) = &options.after {
        match after {
            CommandAfter::Command { id, condition } => {
                command.args(["--property", &format!("After={id}")]);
                if let Some(condition) = condition {
                    match condition {
                        CommandAfterCondition::Always => {}
                        CommandAfterCondition::Success => {
                            command.args(["--property", &format!("Requires={id}")]);
                        }
                    };
                }
            }
            CommandAfter::Date { date } => {
                command.args(["--on-calendar", &format!("@{date}")]);
            }
        };
    }

    for (key, value) in base_command.get_envs() {
        if let Some(value) = value {
            command
                .arg("--setenv")
                .arg([key, value].join(OsStr::new("=")));
        }
    }
    base_command.env_clear();

    let program = base_command.get_program();
    command.arg(program).args(base_command.get_args());

    execute_command_simple(command).await
}

pub fn get_wrapped_unit(name: &str) -> String {
    format!("command-{name}.service")
}
