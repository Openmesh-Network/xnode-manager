use std::{fmt::Display, path::Path};

use tokio::process::Command;

use crate::common::{
    command::{CommandOptions, execute_command_scoped, execute_command_simple},
    env::nix,
    path::get_scoped_path,
    response::{ResponseError, ResponseResult},
};

use super::models::{CliFlakeMetadata, FlakeMetadata};

pub enum Operation {
    Update,
    Build,
    Apply,
}
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operation::Update => "update",
                Operation::Build => "build",
                Operation::Apply => "apply",
            }
        )
    }
}

pub async fn build<SCOPE: AsRef<str>, PATH: AsRef<str>, CHROOT: AsRef<str>>(
    scope: &[SCOPE],
    path: &[PATH],
    chroot: Option<&[CHROOT]>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let mut command = Command::new(format!("{}nix", nix()));
    command.args(["build", "--out-link"]).arg(
        get_scoped_path(scope, path)
            .parent()
            .unwrap_or(Path::new("/"))
            .join("new-result"),
    );

    alter_flake(
        command,
        Operation::Build,
        "#nixosConfigurations.xnode.config.system.build.toplevel",
        scope,
        path,
        chroot,
        options,
    )
    .await
}

pub async fn update<INPUTS: AsRef<str>, SCOPE: AsRef<str>, PATH: AsRef<str>, CHROOT: AsRef<str>>(
    inputs: &[INPUTS],
    scope: &[SCOPE],
    path: &[PATH],
    chroot: Option<&[CHROOT]>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let mut command = Command::new(format!("{}nix", nix()));
    command
        .args(["flake", "update"])
        .args(inputs.iter().map(|s| s.as_ref()))
        .arg("--flake");

    alter_flake(command, Operation::Update, "", scope, path, chroot, options).await
}

pub async fn flake_metadata(flake: &str) -> ResponseResult<FlakeMetadata> {
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

pub async fn eval(statement: &str) -> ResponseResult<String> {
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

async fn alter_flake<SCOPE: AsRef<str>, PATH: AsRef<str>, CHROOT: AsRef<str>>(
    mut command: Command,
    operation: Operation,
    suffix: &str,
    scope: &[SCOPE],
    path: &[PATH],
    chroot: Option<&[CHROOT]>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let path = get_scoped_path(scope, path);

    if let Some(chroot) = chroot {
        let chroot = get_scoped_path(scope, chroot);
        let in_chroot_path = path.strip_prefix(&chroot).map_err(|e| {
            ResponseError::new(format!(
                "Couldn't strip chroot {chroot} from {path}: {e}",
                chroot = chroot.display(),
                path = path.display()
            ))
        })?;
        let flake = format!(
            "./{in_chroot_path}{suffix}",
            in_chroot_path = in_chroot_path.to_string_lossy()
        );
        command.arg(&flake);
        command.env("HOME", "/tmp");
        execute_command_scoped(
            command,
            &operation.to_string(),
            scope,
            Some(chroot),
            None::<String>,
            options,
        )
        .await
        .map(|_output| ())
        .map_err(|e| ResponseError::new(format!("Could not {operation} {flake}: {e}")))
    } else {
        let flake = format!("{path}{suffix}", path = path.to_string_lossy());
        command.arg(&flake);
        command.env("NIX_REMOTE", "daemon");
        execute_command_scoped(
            command,
            &operation.to_string(),
            scope,
            None::<String>,
            None::<String>,
            options,
        )
        .await
        .map(|_output| ())
        .map_err(|e| ResponseError::new(format!("Could not {operation} {flake}: {e}")))
    }
}
