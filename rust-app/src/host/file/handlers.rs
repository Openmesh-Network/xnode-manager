use actix_web::{Responder, get, post, web};

use crate::common::{
    file::{
        PathQuery, Permission, ReadFolderOptions, SourceDestinationData, copy, create_folder,
        get_permissions, metadata, r#move, read_file, read_folder, read_link, remove,
        set_permissions, size, write_file,
    },
    response::{ResponseResult, json_response, raw_response},
};

#[get("/metadata")]
async fn metadata_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    metadata(&query.path).await.map(json_response)
}

#[get("/size")]
async fn size_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    size(&query.path).await.map(json_response)
}

#[post("/move")]
async fn move_endpoint(data: web::Json<SourceDestinationData>) -> ResponseResult<impl Responder> {
    r#move(&data.source, &data.destination)
        .await
        .map(raw_response)
}

#[post("/remove")]
async fn remove_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    remove(&query.path).await.map(raw_response)
}

#[get("/copy")]
async fn copy_endpoint(data: web::Json<SourceDestinationData>) -> ResponseResult<impl Responder> {
    copy(&data.source, &data.destination)
        .await
        .map(raw_response)
}

#[get("/read_file")]
async fn read_file_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    read_file(&query.path).await.map(raw_response)
}

#[post("/write_file")]
async fn write_file_endpoint(
    query: web::Query<PathQuery>,
    data: web::Bytes,
) -> ResponseResult<impl Responder> {
    write_file(&query.path, &data).await.map(raw_response)
}

#[get("/read_folder")]
async fn read_folder_endpoint(
    query: web::Query<PathQuery>,
    options: web::Query<ReadFolderOptions>,
) -> ResponseResult<impl Responder> {
    read_folder(&query.path, &options).await.map(json_response)
}

#[post("/create_folder")]
async fn create_folder_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    create_folder(&query.path).await.map(raw_response)
}

#[get("/read_link")]
async fn read_link_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    read_link(&query.path)
        .await
        .map(|path| path.into_os_string())
        .map(json_response)
}

#[get("/get_permissions")]
async fn get_permissions_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    get_permissions(&query.path).await.map(json_response)
}

#[post("/set_permissions")]
async fn set_permissions_endpoint(
    query: web::Query<PathQuery>,
    data: web::Json<Vec<Permission>>,
) -> ResponseResult<impl Responder> {
    set_permissions(&query.path, data.into_inner())
        .await
        .map(raw_response)
}
