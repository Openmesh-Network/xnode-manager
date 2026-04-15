use actix_web::{Responder, get, post, rt::spawn, web};
use tokio::process::Command;

use crate::{
    common::{
        command::{CommandOptions, ResponseCommand, execute_command_scoped, get_scope_unit},
        file::{r#move, read_file, write_file},
        nix::{ApplyQuery, ApplyWhen, Operation, UpdateData, build, update},
        path::get_scoped_path,
        response::{ResponseError, ResponseResult, json_response, raw_response},
    },
    container::handlers::ensure_initialized,
};

#[get("/get")]
async fn get_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["data", "config", "flake.nix"]);
    read_file(path).await.map(raw_response)
}

#[post("/set")]
async fn set_endpoint(path: web::Path<String>, data: web::Bytes) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["data", "config", "flake.nix"]);
    write_file(path, &data).await.map(raw_response)
}

#[get("/version")]
async fn version_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["data", "config", "flake.lock"]);
    read_file(path).await.map(raw_response)
}

#[post("/update")]
async fn update_endpoint(
    path: web::Path<String>,
    data: web::Json<UpdateData>,
    options: web::Json<CommandOptions>,
) -> impl Responder {
    let container = path.into_inner();
    let scope = ["container".to_string(), container.to_string()];
    let data = data.into_inner();
    let options = options.into_inner();
    let unit = get_scope_unit(&Operation::Update.to_string(), &scope);

    spawn(async move {
        update(
            &data.inputs,
            &scope,
            &["data", "config"],
            Some(&["data"]),
            options,
        )
        .await
    });

    json_response(ResponseCommand { id: unit })
}

#[post("/build")]
async fn build_endpoint(
    path: web::Path<String>,
    options: web::Json<CommandOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container".to_string(), container.to_string()];
    let options = options.into_inner();
    let unit = get_scope_unit(&Operation::Build.to_string(), &scope);

    let xnode_config = get_scoped_path(&scope, &["data", "config", "xnode-config"]);
    write_file(
        xnode_config.join("host-platform"),
        format!("{arch}-linux", arch = std::env::consts::ARCH),
    )
    .await?;
    write_file(xnode_config.join("name"), &container).await?;
    write_file(xnode_config.join("type"), "container").await?;

    spawn(async move { build(&scope, &["data", "config"], Some(&["data"]), options).await });

    Ok(json_response(ResponseCommand { id: unit }))
}

#[post("/apply")]
async fn apply_endpoint(
    path: web::Path<String>,
    query: web::Query<ApplyQuery>,
    options: web::Json<CommandOptions>,
) -> impl Responder {
    let container = path.into_inner();
    let scope = ["container".to_string(), container.to_string()];
    let operation = Operation::Apply.to_string();
    let query = query.into_inner();
    let options = options.into_inner();
    let unit = get_scope_unit(&operation, &scope);

    spawn(async move {
        r#move(
            get_scoped_path(&scope, &["data", "new-result"]),
            get_scoped_path(&scope, &["data", "result"]),
        )
        .await?;

        match &query.when.unwrap_or(ApplyWhen::Now) {
            ApplyWhen::Now => {
                let mut command = Command::new("/result/bin/switch-to-configuration");
                command.arg("switch");

                execute_command_scoped(
                    command,
                    &operation,
                    &scope,
                    None::<String>,
                    Some(format!("{container}.container")),
                    options,
                )
                .await
                .map_err(|e| {
                    ResponseError::new(format!("Could not apply configuration to {container}: {e}"))
                })
            }
            ApplyWhen::NextBoot => {
                let mut command: Command = Command::new("/result/bin/switch-to-configuration");
                command.arg("boot");

                execute_command_scoped(
                    command,
                    &operation,
                    &scope,
                    Some(get_scoped_path(&scope, &["data"])),
                    None::<String>,
                    options,
                )
                .await
                .map_err(|e| {
                    ResponseError::new(format!("Could not apply configuration to {container}: {e}"))
                })
            }
        }
    });

    json_response(ResponseCommand { id: unit })
}
