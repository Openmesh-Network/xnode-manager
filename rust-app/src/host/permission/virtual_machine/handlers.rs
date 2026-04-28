use actix_web::{Responder, get, post, web};

use crate::{
    common::response::{ResponseResult, json_response, raw_response},
    host::permission::{
        handlers::{get_permission, set_permission},
        models::{Permission, SetQuery},
    },
};

#[get("/get")]
async fn get_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let virtual_machine = path.into_inner();
    get_permission("virtual-machine", &virtual_machine)
        .await
        .map(json_response)
}

#[post("/set")]
async fn set_endpoint(
    path: web::Path<String>,
    query: web::Query<SetQuery>,
    data: web::Json<Permission>,
) -> ResponseResult<impl Responder> {
    let virtual_machine = path.into_inner();
    set_permission(
        data.into_inner(),
        "virtual-machine",
        &virtual_machine,
        query.detect_changes.unwrap_or(true),
        query.allow_restart.unwrap_or(true),
    )
    .await
    .map(raw_response)
}
