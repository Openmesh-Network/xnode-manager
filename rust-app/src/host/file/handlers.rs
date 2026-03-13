use actix_web::{Responder, get, post, web};

use super::models::{
    CreateFolderQuery, GetPermissionsQuery, ReadFileQuery, ReadFolderQuery, RemoveFileQuery,
    RemoveFolderQuery, SetPermissionsQuery, WriteFileQuery,
};
use crate::{
    common::{
        file::handlers as fs,
        response::{wrap_json_response, wrap_raw_response},
    },
    host::file::models::{CopyFileData, CopyFolderData, MetadataQuery, SetPermissionsData},
};

#[get("/metadata")]
async fn metadata(query: web::Query<MetadataQuery>) -> impl Responder {
    wrap_json_response(fs::metadata(&query.path).await)
}

#[get("/read_file")]
async fn read_file(query: web::Query<ReadFileQuery>) -> impl Responder {
    wrap_raw_response(fs::read_file(&query.path).await)
}

#[post("/write_file")]
async fn write_file(query: web::Json<WriteFileQuery>, data: web::Bytes) -> impl Responder {
    wrap_raw_response(fs::write_file(&query.path, &data).await)
}

#[post("/remove_file")]
async fn remove_file(query: web::Json<RemoveFileQuery>) -> impl Responder {
    wrap_raw_response(fs::remove_file(&query.path).await)
}

#[get("/copy_file")]
async fn copy_file(data: web::Json<CopyFileData>) -> impl Responder {
    wrap_raw_response(fs::copy_file(&data.source, &data.destination).await)
}

#[get("/read_folder")]
async fn read_folder(query: web::Query<ReadFolderQuery>) -> impl Responder {
    wrap_json_response(fs::read_folder(&query.path).await)
}

#[post("/create_folder")]
async fn create_folder(query: web::Json<CreateFolderQuery>) -> impl Responder {
    wrap_raw_response(fs::create_folder(&query.path).await)
}

#[post("/remove_folder")]
async fn remove_folder(query: web::Json<RemoveFolderQuery>) -> impl Responder {
    wrap_raw_response(fs::remove_folder(&query.path).await)
}

#[get("/copy_folder")]
async fn copy_folder(data: web::Json<CopyFolderData>) -> impl Responder {
    wrap_raw_response(fs::copy_folder(&data.source, &data.destination).await)
}

#[get("/get_permissions")]
async fn get_permissions(query: web::Query<GetPermissionsQuery>) -> impl Responder {
    wrap_json_response(fs::get_permissions(&query.path).await)
}

#[post("/set_permissions")]
async fn set_permissions(
    query: web::Query<SetPermissionsQuery>,
    data: web::Json<SetPermissionsData>,
) -> impl Responder {
    wrap_raw_response(fs::set_permissions(&query.path, data.into_inner()).await)
}
