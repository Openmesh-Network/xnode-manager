use actix_web::{Responder, get, post, web};

use crate::common::{
    file::{
        PathQuery, Permission, SourceDestinationData, copy_file, copy_folder, create_folder,
        get_permissions, metadata, r#move, read_file, read_folder, remove_file, remove_folder,
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

#[get("/read_file")]
async fn read_file_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    read_file(&query.path).await.map(raw_response)
}

#[post("/write_file")]
async fn write_file_endpoint(
    query: web::Json<PathQuery>,
    data: web::Bytes,
) -> ResponseResult<impl Responder> {
    write_file(&query.path, &data).await.map(raw_response)
}

#[post("/remove_file")]
async fn remove_file_endpoint(query: web::Json<PathQuery>) -> ResponseResult<impl Responder> {
    remove_file(&query.path).await.map(raw_response)
}

#[get("/copy_file")]
async fn copy_file_endpoint(
    data: web::Json<SourceDestinationData>,
) -> ResponseResult<impl Responder> {
    copy_file(&data.source, &data.destination)
        .await
        .map(raw_response)
}

#[get("/read_folder")]
async fn read_folder_endpoint(query: web::Query<PathQuery>) -> ResponseResult<impl Responder> {
    read_folder(&query.path).await.map(json_response)
}

#[post("/create_folder")]
async fn create_folder_endpoint(query: web::Json<PathQuery>) -> ResponseResult<impl Responder> {
    create_folder(&query.path).await.map(raw_response)
}

#[post("/remove_folder")]
async fn remove_folder_endpoint(query: web::Json<PathQuery>) -> ResponseResult<impl Responder> {
    remove_folder(&query.path).await.map(raw_response)
}

#[post("/copy_folder")]
async fn copy_folder_endpoint(
    data: web::Json<SourceDestinationData>,
) -> ResponseResult<impl Responder> {
    copy_folder(&data.source, &data.destination)
        .await
        .map(raw_response)
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
