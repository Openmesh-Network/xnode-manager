use actix_web::{Responder, get, web};

use crate::common::{
    info::{get_groups, get_users},
    nix::{eval, flake_metadata},
    response::wrap_json_response,
};

use super::models::{EvalQuery, FlakeQuery};

#[get("/flake/metadata")]
async fn flake_metadata_endpoint(query: web::Query<FlakeQuery>) -> impl Responder {
    wrap_json_response(flake_metadata(&query.flake).await)
}

#[get("/eval")]
async fn eval_endpoint(query: web::Query<EvalQuery>) -> impl Responder {
    wrap_json_response(eval(&query.statement).await)
}

#[get("/users/users")]
async fn users_users_endpoint() -> impl Responder {
    wrap_json_response(get_users(None).await)
}

#[get("/users/groups")]
async fn users_groups_endpoint() -> impl Responder {
    wrap_json_response(get_groups(None).await)
}
