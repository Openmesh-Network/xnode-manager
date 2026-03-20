use actix_web::{HttpResponse, Responder, get, post, rt::spawn, web};
use tokio::process::Command;

use crate::common::{
    command::{ResponseCommand, execute_command_scoped, get_scope_unit},
    error::ResponseError,
    file::{read_file, write_file},
    nix::{Operation, build, update},
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
async fn update_endpoint(data: web::Json<UpdateData>) -> impl Responder {
    let scope = ["host"];
    let data = data.into_inner();
    let unit = get_scope_unit(&Operation::Update.to_string(), &scope);

    spawn(async move { update(&data.inputs, &scope, &["config"], false).await });

    HttpResponse::Ok().json(ResponseCommand { unit })
}

#[post("/apply")]
async fn apply_endpoint() -> impl Responder {
    let scope = ["host"];
    let unit = get_scope_unit(&Operation::Build.to_string(), &scope);

    spawn(async move {
        build(&scope, &["config"], false).await?;

        let path = get_scoped_path(&scope, &["result", "bin", "switch-to-configuration"]);
        let mut command = Command::new(path);
        command.arg("switch");

        execute_command_scoped(command, "apply", &scope, None::<String>)
            .await
            .map_err(|e| ResponseError::new(format!("Could not apply configuration to host: {e}")))
    });

    HttpResponse::Ok().json(ResponseCommand { unit })
}
