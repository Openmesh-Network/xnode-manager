use actix_web::{Responder, get, web};

use crate::{
    common::{
        info::{get_groups, get_users},
        nix::{EvalQuery, FlakeQuery, eval, flake_metadata},
        response::{ResponseResult, json_response},
    },
    host::handlers::{flake, machine},
};

#[get("/flake/metadata")]
async fn flake_metadata_endpoint(query: web::Query<FlakeQuery>) -> ResponseResult<impl Responder> {
    flake_metadata(&query.flake, machine())
        .await
        .map(json_response)
}

#[get("/eval")]
async fn eval_endpoint(query: web::Query<EvalQuery>) -> ResponseResult<impl Responder> {
    let mut statement = query.statement.clone();

    if query.config.unwrap_or(false) {
        statement = format!(
            "{flake}#nixosConfiguration.xnode.{statement}",
            flake = flake().as_ref().to_string_lossy()
        );
    }

    eval(&statement, machine()).await.map(json_response)
}

#[get("/users/users")]
async fn users_users_endpoint() -> ResponseResult<impl Responder> {
    get_users(None::<String>).await.map(json_response)
}

#[get("/users/groups")]
async fn users_groups_endpoint() -> ResponseResult<impl Responder> {
    get_groups(None::<String>).await.map(json_response)
}
