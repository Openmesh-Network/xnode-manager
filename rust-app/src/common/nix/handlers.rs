use std::{fmt::Display, path::Path};

use tokio::process::Command;

use crate::common::{
    command::{CommandOptions, execute_command_simple, execute_command_wrapped},
    env::nix,
    file::r#move,
    response::{ResponseError, ResponseResult},
};

use super::{
    ApplyWhen,
    models::{CliFlakeMetadata, FlakeMetadata},
};

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

pub async fn build(
    flake: impl AsRef<Path>,
    machine: Option<impl AsRef<str>>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let flake = flake.as_ref();

    let mut command = Command::new(format!("{}nix", nix()));
    let out_link = flake.parent().unwrap_or(Path::new("/")).join("new-result");
    command
        .env("NIX_REMOTE", "daemon")
        .arg("build")
        .arg(format!(
            "{flake}#nixosConfigurations.xnode.config.system.build.toplevel",
            flake = flake.to_string_lossy()
        ))
        .arg("--out-link")
        .arg(out_link);

    execute_command_wrapped(command, &Operation::Build.to_string(), machine, options)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not build {flake}: {e}",
                flake = flake.display()
            ))
        })
}

pub async fn update<INPUTS: AsRef<str>>(
    inputs: &[INPUTS],
    flake: impl AsRef<Path>,
    machine: Option<impl AsRef<str>>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let flake = flake.as_ref();

    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .args(["flake", "update"])
        .args(inputs.iter().map(|s| s.as_ref()))
        .arg("--flake")
        .arg(flake);

    execute_command_wrapped(command, &Operation::Update.to_string(), machine, options)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not update {flake}: {e}",
                flake = flake.display()
            ))
        })
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

pub async fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .arg("copy")
        .arg(source)
        .arg("--to")
        .arg(destination)
        .args(["--no-require-sigs"]);
    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not copy {source} to nix root {destination}: {e}",
                source = source.display(),
                destination = destination.display(),
            ))
        })
}

pub async fn switch_to_configuration(
    when: ApplyWhen,
    root: impl AsRef<Path>,
    machine: Option<impl AsRef<str>>,
    options: impl AsRef<CommandOptions>,
) -> ResponseResult<()> {
    let root = root.as_ref();

    r#move(root.join("new-result"), root.join("result")).await?;

    let mut command = Command::new("/result/bin/switch-to-configuration");
    command.arg(match when {
        ApplyWhen::Now => "switch",
        ApplyWhen::NextBoot => "boot",
    });

    execute_command_wrapped(command, &Operation::Apply.to_string(), machine, options)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not apply configuration to {root}: {e}",
                root = root.display()
            ))
        })
}
