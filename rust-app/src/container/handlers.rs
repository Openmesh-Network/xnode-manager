use std::path::Path;

use actix_web::{Responder, post, web};

use crate::{
    common::{
        btrfs::{quota, subvolume},
        env::{build_base, default_permission},
        file::{metadata, write_link},
        nix,
        path::get_scope_root,
        process::{SystemCtlCommand, execute},
        response::{ResponseError, ResponseResult, TypedResponseError, raw_response},
    },
    host::permission::handlers::{get_permission, set_permission},
};

pub fn machine(container: impl AsRef<str>) -> Option<impl AsRef<str>> {
    let container = container.as_ref();
    Some(format!("{container}.container"))
}

pub fn flake() -> impl AsRef<Path> {
    "/config"
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

    nix::copy(build_base(), &data_root).await?;
    write_link(build_base(), data_root.join("result")).await?;

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
                ResponseError::new(format!("Container {container} hasn't been created yet/"))
            } else {
                e
            }
        })
}
