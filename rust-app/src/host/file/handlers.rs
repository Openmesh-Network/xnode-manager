use actix_web::{Responder, get, post, web};

use super::models::{
    CreateFolderQuery, GetPermissionsQuery, MoveData, ReadFileQuery, ReadFolderQuery,
    RemoveFileQuery, RemoveFolderQuery, SetPermissionsQuery, WriteFileQuery,
};
use crate::{
    common::{
        file::{
            copy_file, copy_folder, create_folder, get_permissions, metadata, r#move, read_file,
            read_folder, remove_file, remove_folder, set_permissions, size, write_file,
        },
        response::{wrap_json_response, wrap_raw_response},
    },
    host::file::models::{
        CopyFileData, CopyFolderData, MetadataQuery, SetPermissionsData, SizeQuery,
    },
};

#[get("/metadata")]
async fn metadata_endpoint(query: web::Query<MetadataQuery>) -> impl Responder {
    wrap_json_response(metadata(&query.path).await)
}

#[get("/size")]
async fn size_endpoint(query: web::Query<SizeQuery>) -> impl Responder {
    wrap_json_response(size(&query.path).await)
}

#[post("/move")]
async fn move_endpoint(data: web::Json<MoveData>) -> impl Responder {
    wrap_raw_response(r#move(&data.source, &data.destination).await)
}

#[get("/read_file")]
async fn read_file_endpoint(query: web::Query<ReadFileQuery>) -> impl Responder {
    wrap_raw_response(read_file(&query.path).await)
}

#[post("/write_file")]
async fn write_file_endpoint(query: web::Json<WriteFileQuery>, data: web::Bytes) -> impl Responder {
    wrap_raw_response(write_file(&query.path, &data).await)
}

#[post("/remove_file")]
async fn remove_file_endpoint(query: web::Json<RemoveFileQuery>) -> impl Responder {
    wrap_raw_response(remove_file(&query.path).await)
}

#[get("/copy_file")]
async fn copy_file_endpoint(data: web::Json<CopyFileData>) -> impl Responder {
    wrap_raw_response(copy_file(&data.source, &data.destination).await)
}

#[get("/read_folder")]
async fn read_folder_endpoint(query: web::Query<ReadFolderQuery>) -> impl Responder {
    wrap_json_response(read_folder(&query.path).await)
}

#[post("/create_folder")]
async fn create_folder_endpoint(query: web::Json<CreateFolderQuery>) -> impl Responder {
    wrap_raw_response(create_folder(&query.path).await)
}

#[post("/remove_folder")]
async fn remove_folder_endpoint(query: web::Json<RemoveFolderQuery>) -> impl Responder {
    wrap_raw_response(remove_folder(&query.path).await)
}

#[post("/copy_folder")]
async fn copy_folder_endpoint(data: web::Json<CopyFolderData>) -> impl Responder {
    wrap_raw_response(copy_folder(&data.source, &data.destination).await)
}

#[get("/get_permissions")]
async fn get_permissions_endpoint(query: web::Query<GetPermissionsQuery>) -> impl Responder {
    wrap_json_response(get_permissions(&query.path).await)
}

#[post("/set_permissions")]
async fn set_permissions_endpoint(
    query: web::Query<SetPermissionsQuery>,
    data: web::Json<SetPermissionsData>,
) -> impl Responder {
    wrap_raw_response(set_permissions(&query.path, data.into_inner()).await)
}
