use actix_web::{dev::HttpServiceFactory, rt::spawn, web};

use crate::common::{
    btrfs::subvolume,
    env::datadir,
    file::{ReadFolderOptions, metadata, read_folder},
    process::execute,
    response::{ResponseError, ResponseResult, TypedResponseError},
};

pub mod backup;
pub mod config;
pub mod file;
pub mod handlers;
pub mod info;
pub mod models;
pub mod process;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/container").configure(|cfg| {
        cfg.service(handlers::container_endpoint);
        cfg.service(
            // Container name can use lowercase letters, numbers, and - (dash)
            // Container name must be minimum 1 and maximum 32 characters
            web::scope("/{container:[a-z0-9-]{1,32}}")
                .configure(|cfg| {
                    cfg.service(handlers::create_endpoint);
                    cfg.service(handlers::remove_endpoint);
                })
                .service(backup::service())
                .service(config::service())
                .service(file::service())
                .service(info::service())
                .service(process::service()),
        );
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
        let containers = read_folder(path, &ReadFolderOptions::default()).await?;
        for item in containers {
            let container = item.name;
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
