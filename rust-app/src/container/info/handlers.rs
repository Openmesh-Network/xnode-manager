use actix_web::{Responder, get, web};

use crate::{
    common::{
        info::{get_groups, get_users},
        nix::{EvalQuery, FlakeQuery, eval, flake_metadata},
        path::get_scope_root,
        response::{ResponseResult, json_response},
    },
    container::handlers::{flake, machine},
};

#[get("/flake/metadata")]
async fn flake_metadata_endpoint(
    path: web::Path<String>,
    query: web::Query<FlakeQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    flake_metadata(&query.flake, machine(&container))
        .await
        .map(json_response)
}

#[get("/eval")]
async fn eval_endpoint(
    path: web::Path<String>,
    query: web::Query<EvalQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let mut statement = query.statement.clone();

    if query.config.unwrap_or(false) {
        statement = format!(
            "{flake}#nixosConfigurations.xnode.{statement}",
            flake = flake().as_ref().to_string_lossy()
        );
    }

    eval(&statement, machine(&container))
        .await
        .map(json_response)
}

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
