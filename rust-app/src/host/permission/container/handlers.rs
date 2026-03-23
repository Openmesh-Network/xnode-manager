use actix_web::{Responder, get, post, web};

use super::models::{SetData, SetQuery};
use crate::{
    common::{
        path::get_scoped_path,
        response::{wrap_json_response, wrap_raw_response},
    },
    host::permission::handlers::{get_permission, set_permission},
};

#[get("/{container}/get")]
async fn get_endpoint(path: web::Path<String>) -> impl Responder {
    let container = path.into_inner();
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["permission", "container", "config", &container]);
    wrap_json_response(get_permission(path).await)
}

#[post("/{container}/set")]
async fn set_endpoint(
    path: web::Path<String>,
    query: web::Query<SetQuery>,
    data: web::Json<SetData>,
) -> impl Responder {
    let container = path.into_inner();
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["permission", "container", "config", &container]);
    wrap_raw_response(set_permission(path, data.into_inner(), true, query.allow_restart).await)
}
