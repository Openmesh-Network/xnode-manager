use actix_web::{HttpResponse, Responder, get, web};
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::nix,
    error::ResponseError,
    info::handlers::{get_groups, get_users},
};

use super::models::{EvalQuery, Flake, FlakeMetadata, FlakeQuery};

#[get("/flake")]
async fn flake(query: web::Query<FlakeQuery>) -> impl Responder {
    let mut command = Command::new(format!("{}nix", nix()));
    command.env("NIX_REMOTE", "daemon").args([
        "flake",
        "metadata",
        &query.flake,
        "--json",
        "--no-use-registries",
        "--refresh",
        "--no-write-lock-file",
    ]);

    match execute_command_simple(command).await {
        Ok(output) => match String::from_utf8(output) {
            Ok(output_str) => match serde_json::from_str::<FlakeMetadata>(&output_str) {
                Ok(output_parsed) => HttpResponse::Ok().json(Flake {
                    last_modified: output_parsed.lastModified,
                    revision: output_parsed.revision,
                }),
                Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Flake metadata could not be parsed to expected format: {e}. Metadata: {output_str}"
                ))),
            },
            Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Flake metadata could not be decoded as UTF8: {e}."
            ))),
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error getting flake metadata of {flake}: {e}",
            flake = &query.flake
        ))),
    }
}

#[get("/eval")]
async fn eval(query: web::Query<EvalQuery>) -> impl Responder {
    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .args(["eval", &query.statement]);

    match execute_command_simple(command).await {
        Ok(output) => match String::from_utf8(output) {
            Ok(output_str) => HttpResponse::Ok().json(output_str),
            Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Eval result could not be decoded as UTF8: {e}."
            ))),
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error evaluating {statement}: {e}",
            statement = &query.statement
        ))),
    }
}

#[get("/users/users")]
async fn users() -> impl Responder {
    match get_users(None).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

#[get("/users/groups")]
async fn groups() -> impl Responder {
    match get_groups(None).await {
        Ok(groups) => HttpResponse::Ok().json(groups),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
