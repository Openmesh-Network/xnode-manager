use actix_web::{dev::HttpServiceFactory, rt::spawn, web};

use crate::{
    common::{
        btrfs::{quota, subvolume},
        env::datadir,
        file::{metadata, read_folder},
        path::get_scope_root,
        process::execute,
        response::{ResponseError, ResponseResult, TypedResponseError},
    },
    host::permission::handlers::{get_permission, set_permission},
};

pub mod config;
pub mod file;
pub mod info;
pub mod process;

pub fn scope() -> String {
    // Container name can use lowercase letters, numbers, and - (dash)
    // Container name must be minimum 1 and maximum 32 characters
    "/container/{container:[a-z0-9-]{1,32}}".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope())
        .service(config::service())
        .service(file::service())
        .service(info::service())
        .service(process::service())
}

pub async fn prepare_module() -> ResponseResult<()> {
    let path = datadir().join("container");

    if let Err(e) = metadata(&path).await {
        if let Some(typed) = &e.typed_error
            && matches!(typed, TypedResponseError::PathNotFound { path: _path })
        {
            subvolume::create(&path).await?;
        } else {
            return Err(e);
        }
    };

    spawn(async {
        // start all containers
        let containers = read_folder(path).await.map(|folder| folder.folders)?;
        for container in containers {
            if let Err(e) = execute(
                None::<String>,
                format!("container@{container}.service"),
                crate::common::process::SystemCtlCommand::Start,
            )
            .await
            {
                log::warn!("Could not start container {container}: {e}");
            }
        }

        Ok::<(), ResponseError>(())
    });

    Ok(())
}

pub async fn ensure_initialized(container: impl AsRef<str>) -> ResponseResult<()> {
    let container = container.as_ref();
    let scope = ["container", container];

    if let Err(e) = metadata(get_scope_root(&scope)).await {
        if let Some(typed) = &e.typed_error
            && matches!(typed, TypedResponseError::PathNotFound { path: _path })
        {
            // Container doesn't exist yes, initialize
            initialize_container(&container).await?;
        } else {
            return Err(e);
        }
    };

    Ok(())
}

async fn initialize_container(container: impl AsRef<str>) -> ResponseResult<()> {
    let kind = "container";
    let container = container.as_ref();
    let scope = [kind, container];
    let root = get_scope_root(&scope);
    subvolume::create(&root).await?;
    quota::enable(&root).await?;
    subvolume::create(&root.join("data")).await?;

    let permission = get_permission(kind, container).await.or_else(|e| {
        if let Some(typed) = &e.typed_error
            && matches!(typed, TypedResponseError::PathNotFound { path: _path })
        {
            // Replace file not found with default Permission
            return Ok(crate::host::permission::models::Permission {
                process: None,
                disk: None,
                bind: None,
                device: None,
                extra_args: None,
            });
        }

        Err(e)
    })?;
    set_permission(permission, kind, container, false, false).await?;

    Ok(())
}
