use std::path::PathBuf;

use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        file::{
            PathQuery, Permission, SourceDestinationData, copy_file, copy_folder, create_folder,
            get_permissions, metadata, r#move, read_file, read_folder, remove_file,
            remove_first_slash, remove_folder, set_permissions, size, write_file,
        },
        path::get_scope_root,
        response::{ResponseError, ResponseResult, json_response, raw_response},
    },
    container::ensure_initialized,
};

#[get("/metadata")]
async fn metadata_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    metadata(path).await.map(json_response)
}

#[get("/size")]
async fn size_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    size(path).await.map(json_response)
}

#[post("/move")]
async fn move_endpoint(
    path: web::Path<String>,
    data: web::Json<SourceDestinationData>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let source = to_container_path(&container, &data.source)?;
    let destination = to_container_path(&container, &data.destination)?;
    r#move(source, destination).await.map(raw_response)
}

#[get("/read_file")]
async fn read_file_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    read_file(path).await.map(raw_response)
}

#[post("/write_file")]
async fn write_file_endpoint(
    path: web::Path<String>,
    query: web::Json<PathQuery>,
    data: web::Bytes,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let path = to_container_path(&container, &query.path)?;
    write_file(path, &data).await.map(raw_response)
}

#[post("/remove_file")]
async fn remove_file_endpoint(
    path: web::Path<String>,
    query: web::Json<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    remove_file(path).await.map(raw_response)
}

#[get("/copy_file")]
async fn copy_file_endpoint(
    path: web::Path<String>,
    data: web::Json<SourceDestinationData>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let source = to_container_path(&container, &data.source)?;
    let destination = to_container_path(&container, &data.destination)?;
    copy_file(source, destination).await.map(raw_response)
}

#[get("/read_folder")]
async fn read_folder_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    read_folder(path).await.map(json_response)
}

#[post("/create_folder")]
async fn create_folder_endpoint(
    path: web::Path<String>,
    query: web::Json<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let path = to_container_path(&container, &query.path)?;
    create_folder(path).await.map(raw_response)
}

#[post("/remove_folder")]
async fn remove_folder_endpoint(
    path: web::Path<String>,
    query: web::Json<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    remove_folder(path).await.map(raw_response)
}

#[post("/copy_folder")]
async fn copy_folder_endpoint(
    path: web::Path<String>,
    data: web::Json<SourceDestinationData>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let source = to_container_path(&container, &data.source)?;
    let destination = to_container_path(&container, &data.destination)?;
    copy_folder(source, destination).await.map(raw_response)
}

#[get("/get_permissions")]
async fn get_permissions_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    get_permissions(path).await.map(json_response)
}

#[post("/set_permissions")]
async fn set_permissions_endpoint(
    path: web::Path<String>,
    query: web::Query<PathQuery>,
    data: web::Json<Vec<Permission>>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let path = to_container_path(&container, &query.path)?;
    set_permissions(path, data.into_inner())
        .await
        .map(raw_response)
}

fn to_container_path(container: impl AsRef<str>, path: impl AsRef<str>) -> ResponseResult<PathBuf> {
    let container = container.as_ref();
    let path = remove_first_slash(path.as_ref());

    let scope = ["container", container];
    let root = get_scope_root(&scope);
    let container_path = root.join(path);

    container_path.strip_prefix(&root).map_err(|e| {
        ResponseError::new(format!(
            "Container path {container_path} outside of container root {root}: {e}",
            container_path = container_path.display(),
            root = root.display()
        ))
    })?;

    Ok(container_path)
}
