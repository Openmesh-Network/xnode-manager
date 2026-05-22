use std::path::Path;

use futures::future::join_all;
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple_machine,
    env::systemd,
    file::{ReadFolderOptions, read_folder},
    response::{ResponseError, ResponseResult, TypedResponseError},
};

use super::{Secret, SecretOptions};

pub async fn list(
    dir: impl AsRef<Path>,
    machine: Option<impl AsRef<str>>,
    options: SecretOptions,
) -> ResponseResult<Vec<Secret>> {
    let dir = dir.as_ref();

    let secrets = read_folder(dir, &ReadFolderOptions::default())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| item.name)
                .collect::<Vec<String>>()
        })
        .or_else(|e| {
            if let Some(TypedResponseError::PathNotFound { path: _ }) = e.typed_error {
                return Ok(vec![]);
            }

            Err(e)
        })?;

    let mut items = vec![];

    for secret in secrets {
        let machine = machine.as_ref();

        let get = async move {
            let mut secret_get = None;
            if options.get.unwrap_or(false) {
                secret_get = get(dir.join(&secret), machine).await.ok();
            }

            Secret {
                id: secret,
                get: secret_get,
            }
        };

        items.push(get);
    }

    Ok(join_all(items).await)
}

pub async fn get(
    path: impl AsRef<Path>,
    machine: Option<impl AsRef<str>>,
) -> ResponseResult<Vec<u8>> {
    let path = path.as_ref();

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let mut command = Command::new(format!("{}systemd-creds", systemd()));
    command.args(["decrypt"]).arg(path);
    execute_command_simple_machine(command, None::<Vec<u8>>, machine)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not get secret at {path} for {machine_str}: {e}",
                path = path.display()
            ))
        })
}

pub async fn set(
    path: impl AsRef<Path>,
    value: impl AsRef<[u8]>,
    machine: Option<impl AsRef<str>>,
) -> ResponseResult<()> {
    let path = path.as_ref();

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let mut command = Command::new(format!("{}systemd-creds", systemd()));
    command.args(["encrypt", "-"]).arg(path);
    execute_command_simple_machine(command, Some(value), machine)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not set secret at {path} for {machine_str}: {e}",
                path = path.display()
            ))
        })
}
