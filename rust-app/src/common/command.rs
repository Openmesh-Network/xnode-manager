use std::{ffi::OsStr, fmt::Display, io::Error, path::Path};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::common::{
    env::{nix, systemd},
    string::escaped_utf8_from_bytes,
};

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
        date: u64, // Epoch time in seconds
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

pub async fn execute_command_scoped<SCOPE: AsRef<str>>(
    command: Command,
    name: &str,
    scope: &[SCOPE],
    chroot: Option<impl AsRef<Path>>,
    options: impl AsRef<CommandOptions>,
) -> SimpleCommandResult {
    let mut base_command = command.into_std();
    let options = options.as_ref();

    let mut command = Command::new(format!("{}systemd-run", systemd()));
    command.args([
        "--wait",
        "--quiet",
        "--collect",
        "--unit",
        &get_scope_unit(name, scope),
        "--slice",
        &get_scope_slice(name, scope),
    ]);

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

    if let Some(chroot) = &chroot {
        command.arg("--root-directory").arg(chroot.as_ref());
    }

    for (key, value) in base_command.get_envs() {
        if let Some(value) = value {
            command
                .arg(" --setenv")
                .arg([key, value].join(OsStr::new("=")));
        }
    }
    base_command.env_clear();

    let program = base_command.get_program();
    command.arg(program).args(base_command.get_args());

    if let Some(chroot) = &chroot
        && let Some(program) = program.to_str()
        && program.starts_with("/nix/store")
    {
        // Program + dependencies need to be copied over to be available in chroot environment
        let parts = program.split("/");
        let nix_item: String = parts.take(3).collect();
        let mut nix_copy = Command::new(format!("{}nix", nix()));
        nix_copy
            .env("NIX_REMOTE", "daemon")
            .args(["copy", &nix_item, "--to"])
            .arg(chroot.as_ref());
        execute_command_simple(nix_copy).await?;
    }

    execute_command_simple(command).await
}

/// name: build, scope: [container, xnode-manager] -> container-xnode_manager-command-build.service
pub fn get_scope_unit<SCOPE: AsRef<str>>(name: &str, scope: &[SCOPE]) -> String {
    let mut unit = scope
        .iter()
        .map(|s| s.as_ref())
        .chain(["command", name])
        .map(|s| s.replace("-", "_"))
        .collect::<Vec<String>>()
        .join("-");
    unit.push_str(".service");

    unit
}

/// name: build, scope: [container, xnode-manager] -> build-command-xnode_manager-container-machine.slice
pub fn get_scope_slice<SCOPE: AsRef<str>>(name: &str, scope: &[SCOPE]) -> String {
    let mut slice = ["machine"]
        .into_iter()
        .chain(scope.iter().map(|s| s.as_ref()))
        .chain(["command", name])
        .map(|s| s.replace("-", "_"))
        .rev()
        .collect::<Vec<String>>()
        .join("-");
    slice.push_str(".slice");

    slice
}
