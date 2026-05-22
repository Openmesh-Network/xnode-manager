use std::path::PathBuf;

use actix_web::{Responder, get, post, web};
use tokio::process::Command;

use crate::{
    common::{
        btrfs::{quota, subvolume},
        command::execute_command_simple,
        env::{build_base, datadir, default_permission, systemd},
        file::{ReadFolderOptions, metadata, read_folder, shift, write_link},
        nix,
        path::get_scope_root,
        process::{SystemCtlCommand, execute},
        response::{
            ResponseError, ResponseResult, TypedResponseError, json_response, raw_response,
        },
    },
    container::models::Container,
    host::permission::handlers::{get_permission, set_permission},
};

pub fn machine(container: impl AsRef<str>) -> Option<String> {
    let container = container.as_ref();
    Some(format!("{container}.container"))
}

pub fn config_dir() -> PathBuf {
    "/config".into()
}

#[get("")]
async fn container_endpoint() -> ResponseResult<impl Responder> {
    let path = datadir().join("container");
    read_folder(path, &ReadFolderOptions::default())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| Container { id: item.name })
                .collect::<Vec<Container>>()
        })
        .map(json_response)
}

#[post("/create")]
async fn create_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let kind = "container";
    let scope = [kind, &container];
    let root = get_scope_root(&scope);
    let data_root = root.join("data");
    subvolume::create(&root).await?;
    quota::enable(&root).await?;
    subvolume::create(&data_root).await?;

    let permission = get_permission(kind, &container).await.or_else(|e| {
        if let Some(typed) = &e.typed_error
            && matches!(typed, TypedResponseError::PathNotFound { path: _path })
        {
            // Replace file not found with default Permission
            return Ok(default_permission().container);
        }

        Err(e)
    })?;
    set_permission(permission, kind, &container, false, false).await?;

    let build_base = build_base().container;
    nix::copy(&build_base, &data_root).await?;
    write_link(&build_base, data_root.join("new-result")).await?;

    let mut first_install = Command::new(format!("{}systemd-run", systemd()));
    first_install.args([
        "--pipe",
        "--collect",
        "--property",
        "Type=oneshot",
        "--root-directory",
    ]);
    first_install.arg(&data_root);
    first_install.arg("/new-result/first-install");
    execute_command_simple(first_install, None::<Vec<u8>>)
        .await
        .map_err(|e| ResponseError::new(format!("Could not perform first install: {e}")))?;

    shift(&root, "foreign").await?;

    execute(
        None::<String>,
        format!("container@{container}.service"),
        SystemCtlCommand::Start,
    )
    .await?;

    Ok(raw_response(()))
}

#[post("/remove")]
async fn remove_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let scope = ["container", &container];
    let root = get_scope_root(&scope);

    execute(
        None::<String>,
        format!("container@{container}.service"),
        SystemCtlCommand::Stop,
    )
    .await?;
    subvolume::delete(&root).await?;

    Ok(raw_response(()))
}

pub async fn ensure_initialized(container: impl AsRef<str>) -> ResponseResult<()> {
    let container = container.as_ref();
    let scope = ["container", container];

    metadata(get_scope_root(&scope))
        .await
        .map(|_metadata| ())
        .map_err(|e| {
            if let Some(typed) = &e.typed_error
                && matches!(typed, TypedResponseError::PathNotFound { path: _path })
            {
                ResponseError::new(format!("Container {container} hasn't been created yet."))
            } else {
                e
            }
        })
}
