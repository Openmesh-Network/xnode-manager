use actix_web::{Responder, get, web};

use crate::common::{
    info::{get_groups, get_users},
    path::get_scope_root,
    response::{ResponseResult, json_response},
};

#[get("/users/users")]
async fn users_users_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let scope = ["container".to_string(), container];
    let path = get_scope_root(&scope);
    get_users(Some(path)).await.map(json_response)
}

#[get("/users/groups")]
async fn users_groups_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let scope = ["container".to_string(), container];
    let path = get_scope_root(&scope);
    get_groups(Some(path)).await.map(json_response)
}
