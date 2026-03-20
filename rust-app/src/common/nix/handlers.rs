use std::{fmt::Display, path::Path};

use tokio::process::Command;

use crate::common::{
    command::{execute_command_scoped, execute_command_simple},
    env::nix,
    error::ResponseError,
    path::get_scoped_path,
};

use super::models::{CliFlakeMetadata, FlakeMetadata};

pub enum Operation {
    Build,
    Update,
}
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operation::Build => "build",
                Operation::Update => "update",
            }
        )
    }
}

pub async fn build<SCOPE: AsRef<str>, PATH: AsRef<str>>(
    scope: &[SCOPE],
    path: &[PATH],
    chroot: bool,
) -> Result<(), ResponseError> {
    let mut command = Command::new(format!("{}nix", nix()));
    command.args(["build", "--out-link"]).arg(
        get_scoped_path(scope, path)
            .parent()
            .unwrap_or(Path::new("/"))
            .join("result"),
    );

    alter_flake(
        command,
        Operation::Build,
        "#nixosConfigurations.xnode.config.system.build.toplevel",
        scope,
        path,
        chroot,
    )
    .await
}

pub async fn update<INPUTS: AsRef<str>, SCOPE: AsRef<str>, PATH: AsRef<str>>(
    inputs: &[INPUTS],
    scope: &[SCOPE],
    path: &[PATH],
    chroot: bool,
) -> Result<(), ResponseError> {
    let mut command = Command::new(format!("{}nix", nix()));
    command
        .args(["flake", "update"])
        .args(inputs.iter().map(|s| s.as_ref()))
        .arg("--flake");

    alter_flake(command, Operation::Update, "", scope, path, chroot).await
}

pub async fn flake_metadata(flake: &str) -> Result<FlakeMetadata, ResponseError> {
    let mut command = Command::new(format!("{}nix", nix()));
    command.env("NIX_REMOTE", "daemon").args([
        "flake",
        "metadata",
        flake,
        "--json",
        "--no-use-registries",
        "--refresh",
        "--no-write-lock-file",
    ]);

    let output = execute_command_simple(command)
        .await
        .map_err(|e| ResponseError::new(format!("Could not get flake metadata of {flake}: {e}")))?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!("Flake metadata could not be decoded as UTF8: {e}."))
    })?;

    serde_json::from_str::<CliFlakeMetadata>(&output_str)
        .map(|metadata| FlakeMetadata {
            last_modified: metadata.lastModified,
            revision: metadata.revision,
        })
        .map_err(|e| {
            ResponseError::new(format!(
                "Flake metadata could not be parsed to expected format: {e}. Input: {output_str}"
            ))
        })
}

pub async fn eval(statement: &str) -> Result<String, ResponseError> {
    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .args(["eval", statement]);

    let output = execute_command_simple(command)
        .await
        .map_err(|e| ResponseError::new(format!("Could not evaluate {statement}: {e}")))?;

    String::from_utf8(output)
        .map_err(|e| ResponseError::new(format!("Eval result could not be decoded as UTF8: {e}.")))
}

async fn alter_flake<SCOPE: AsRef<str>, PATH: AsRef<str>>(
    mut command: Command,
    operation: Operation,
    suffix: &str,
    scope: &[SCOPE],
    path: &[PATH],
    chroot: bool,
) -> Result<(), ResponseError> {
    let path = get_scoped_path(scope, path);
    let flake = format!("{path}{suffix}", path = path.to_string_lossy());
    command.arg(&flake);

    let result = if chroot {
        command.env("HOME", "/tmp");
        execute_command_scoped(
            command,
            &operation.to_string(),
            scope,
            Some(path.parent().unwrap_or(Path::new("/"))),
        )
        .await
    } else {
        command.env("NIX_REMOTE", "daemon");
        execute_command_scoped(command, &operation.to_string(), scope, None::<String>).await
    };

    result
        .map(|_output| ())
        .map_err(|e| ResponseError::new(format!("Could not {operation} {flake}: {e}")))
}
