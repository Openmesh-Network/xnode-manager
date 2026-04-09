use actix_web::{Responder, post, web};

use crate::{
    common::{
        btrfs::{quota, subvolume},
        file::metadata,
        path::get_scope_root,
        process::{SystemCtlCommand, execute},
        response::{ResponseResult, TypedResponseError, raw_response},
    },
    host::permission::handlers::{get_permission, set_permission},
};

#[post("/remove")]
async fn remove(path: web::Path<String>) -> ResponseResult<impl Responder> {
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
