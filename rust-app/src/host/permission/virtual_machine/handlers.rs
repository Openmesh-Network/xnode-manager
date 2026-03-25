use actix_web::{Responder, get, post, web};

use super::models::{SetData, SetQuery};
use crate::{
    common::response::{wrap_json_response, wrap_raw_response},
    host::permission::handlers::{get_permission, set_permission},
};

#[get("/{virtual_machine}/get")]
async fn get_endpoint(path: web::Path<String>) -> impl Responder {
    let virtual_machine = path.into_inner();
    wrap_json_response(get_permission("virtual-machine", &virtual_machine).await)
}

#[post("/{virtual_machine}/set")]
async fn set_endpoint(
    path: web::Path<String>,
    query: web::Query<SetQuery>,
    data: web::Json<SetData>,
) -> impl Responder {
    let virtual_machine = path.into_inner();
    wrap_raw_response(
        set_permission(
            data.into_inner(),
            "virtual-machine",
            &virtual_machine,
            query.detect_changed.unwrap_or(true),
            query.allow_restart.unwrap_or(true),
        )
        .await,
    )
}
