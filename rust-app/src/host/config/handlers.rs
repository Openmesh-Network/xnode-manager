use actix_web::{HttpResponse, Responder, get, post, rt::spawn, web};
use tokio::process::Command;

use crate::common::{
    command::{CommandOptions, ResponseCommand, execute_command_scoped, get_scope_unit},
    error::ResponseError,
    file::{read_file, write_file},
    nix::{ApplyQuery, ApplyWhen, Operation, build, update},
    path::get_scoped_path,
    response::{wrap_json_response, wrap_raw_response},
};

use super::models::UpdateData;

#[get("/get")]
async fn get_endpoint() -> impl Responder {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.nix"]);
    wrap_json_response(read_file(path).await)
}

#[post("/set")]
async fn set_endpoint(data: web::Bytes) -> impl Responder {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.nix"]);
    wrap_raw_response(write_file(path, &data).await)
}

#[get("/version")]
async fn version_endpoint() -> impl Responder {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.lock"]);
    wrap_json_response(read_file(path).await)
}

#[post("/update")]
async fn update_endpoint(
    data: web::Json<UpdateData>,
    options: web::Query<CommandOptions>,
) -> impl Responder {
    let scope = ["host"];
    let data = data.into_inner();
    let options = options.into_inner();
    let unit = get_scope_unit(&Operation::Update.to_string(), &scope);

    spawn(async move { update(&data.inputs, &scope, &["config"], false, options).await });

    HttpResponse::Ok().json(ResponseCommand { id: unit })
}

#[post("/build")]
async fn build_endpoint(options: web::Query<CommandOptions>) -> impl Responder {
    let scope = ["host"];
    let options = options.into_inner();
    let unit = get_scope_unit(&Operation::Build.to_string(), &scope);

    spawn(async move { build(&scope, &["config"], false, options).await });

    HttpResponse::Ok().json(ResponseCommand { id: unit })
}

#[post("/apply")]
async fn apply_endpoint(
    query: web::Query<ApplyQuery>,
    options: web::Query<CommandOptions>,
) -> impl Responder {
    let scope = ["host"];
    let operation = Operation::Apply.to_string();
    let query = query.into_inner();
    let options = options.into_inner();
    let unit = get_scope_unit(&operation, &scope);

    spawn(async move {
        let path = get_scoped_path(&scope, &["result", "bin", "switch-to-configuration"]);
        let mut command = Command::new(path);
        command.arg(match &query.when {
            Some(when) => match when {
                ApplyWhen::Now => "switch",
                ApplyWhen::NextBoot => "boot",
            },
            None => "switch",
        });

        execute_command_scoped(command, &operation, &scope, None::<String>, options)
            .await
            .map_err(|e| ResponseError::new(format!("Could not apply configuration to host: {e}")))
    });

    HttpResponse::Ok().json(ResponseCommand { id: unit })
}
