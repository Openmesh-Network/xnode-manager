use actix_web::{Responder, get, post, rt::spawn, web};

use crate::{
    common::{
        command::{CommandOptions, ResponseCommand, get_wrapped_unit},
        file::{read_file, write_file},
        nix::{
            ApplyQuery, ApplyWhen, Operation, UpdateData, build, switch_to_configuration, update,
        },
        path::{get_scope_root, get_scoped_path},
        response::{ResponseResult, json_response, raw_response},
    },
    host::handlers::{flake, machine},
};

#[get("/get")]
async fn get_endpoint() -> ResponseResult<impl Responder> {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.nix"]);
    read_file(path).await.map(raw_response)
}

#[post("/set")]
async fn set_endpoint(data: web::Bytes) -> ResponseResult<impl Responder> {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.nix"]);
    write_file(path, &data).await.map(raw_response)
}

#[get("/version")]
async fn version_endpoint() -> ResponseResult<impl Responder> {
    let scope = ["host"];
    let path = get_scoped_path(&scope, &["config", "flake.lock"]);
    read_file(path).await.map(raw_response)
}

#[post("/update")]
async fn update_endpoint(
    data: web::Json<UpdateData>,
    options: web::Json<CommandOptions>,
) -> impl Responder {
    let data = data.into_inner();
    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Update.to_string());

    spawn(async move { update(&data.inputs, flake(), machine(), options).await });

    json_response(ResponseCommand { id: unit })
}

#[post("/build")]
async fn build_endpoint(options: web::Json<CommandOptions>) -> impl Responder {
    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Build.to_string());

    spawn(async move { build(flake(), machine(), options).await });

    json_response(ResponseCommand { id: unit })
}

#[post("/apply")]
async fn apply_endpoint(
    query: web::Query<ApplyQuery>,
    options: web::Json<CommandOptions>,
) -> ResponseResult<impl Responder> {
    let scope = ["host"];
    let query = query.into_inner();
    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Apply.to_string());

    let root = get_scope_root(&scope);

    spawn(async move {
        switch_to_configuration(
            query.when.unwrap_or(ApplyWhen::Now),
            root,
            machine(),
            options,
        )
        .await
    });

    Ok(json_response(ResponseCommand { id: unit }))
}
