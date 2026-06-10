use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        btrfs::subvolume,
        file::{ReadFolderOptions, create_folder, r#move, read_folder, shift},
        path::get_scoped_path,
        process::{SystemCtlCommand, execute},
        response::{ResponseResult, json_response, raw_response},
    },
    container::{backup::models::Backup, handlers::ensure_initialized},
};

#[get("")]
async fn backup_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup"]);
    read_folder(path, &ReadFolderOptions::default())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| Backup { id: item.name })
                .collect::<Vec<Backup>>()
        })
        .map(json_response)
}

#[post("/create")]
async fn create_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let root = get_scoped_path(&scope, &["data"]);
    let snapshot = get_scoped_path(&scope, &["backup", &backup]);

    let parent = get_scoped_path(&scope, &["backup"]);
    create_folder(&parent).await?;
    shift(&parent, "foreign").await?;

    subvolume::snapshot(&root, &snapshot, true)
        .await
        .map(raw_response)
}

#[post("/restore")]
async fn restore_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let root = get_scoped_path(&scope, &["data"]);
    let snapshot = get_scoped_path(&scope, &["backup", &backup]);

    r#move(&root, get_scoped_path(&scope, &["backup", "pre-restore"])).await?;

    subvolume::snapshot(&snapshot, &root, false).await?;

    execute(
        None::<String>,
        format!("container@{container}.service"),
        SystemCtlCommand::Restart,
    )
    .await?;

    Ok(raw_response(()))
}

#[post("/remove")]
async fn remove_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup", &backup]);

    subvolume::delete(&path).await.map(raw_response)
}
