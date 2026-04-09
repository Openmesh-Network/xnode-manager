use actix_web::{dev::HttpServiceFactory, rt::spawn, web};

use crate::common::{
    btrfs::subvolume,
    env::datadir,
    file::{metadata, read_folder},
    process::execute,
    response::{ResponseError, ResponseResult, TypedResponseError},
};

pub mod config;
pub mod file;
pub mod handlers;
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
        .configure(|cfg| {
            cfg.service(handlers::remove);
        })
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
