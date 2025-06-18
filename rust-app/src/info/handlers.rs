use std::process::Command;

use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    info::models::{Flake, FlakeMetadata, FlakeQuery},
    utils::{
        command::{execute_command, CommandExecutionMode},
        env::nix,
        error::ResponseError,
        output::Output,
    },
};

#[get("/flake")]
async fn flake(user: Identity, query: web::Query<FlakeQuery>) -> impl Responder {
    if !has_permission(user, Scope::Info) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .arg("flake")
        .arg("metadata")
        .arg(&query.flake)
        .arg("--json")
        .arg("--no-use-registries")
        .arg("--refresh")
        .arg("--no-write-lock-file");

    match execute_command(command, CommandExecutionMode::Simple) {
        Ok(output) => match output.into() {
            Output::UTF8 { output: output_str } => {
                match serde_json::from_str::<FlakeMetadata>(&output_str) {
                    Ok(output_parsed) => HttpResponse::Ok().json(Flake {
                        last_modified: output_parsed.lastModified,
                        revision: output_parsed.revision,
                    }),
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Flake metadata could not be parsed to expected format: {}. Metadata: {}",
                        e, output_str
                    )))
                    }
                }
            }
            Output::Bytes { output } => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Flake metadata could not be decoded as UTF8: {:?}.",
                    output
                )))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error getting flake metadata of {}: {}",
            &query.flake, e
        ))),
    }
}
