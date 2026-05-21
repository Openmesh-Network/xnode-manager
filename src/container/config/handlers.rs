use actix_web::{Responder, get, post, rt::spawn, web};

use crate::{
    common::{
        command::{CommandOptions, ResponseCommand, get_wrapped_unit},
        file::{read_file, shift, write_file},
        nix::{
            ApplyQuery, ApplyWhen, Operation, UpdateData, build, switch_to_configuration, update,
        },
        path::get_scoped_path,
        response::{ResponseResult, json_response, raw_response},
    },
    container::handlers::{ensure_initialized, flake, machine},
};

#[get("/get")]
async fn get_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

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
    write_file(path, &data).await?;

    let xnode_config = get_scoped_path(&scope, &["data", "config", "xnode-config"]);
    write_file(
        xnode_config.join("host-platform"),
        format!("{arch}-linux", arch = std::env::consts::ARCH),
    )
    .await?;
    write_file(xnode_config.join("name"), &container).await?;
    write_file(xnode_config.join("type"), "container").await?;

    shift(get_scoped_path(&scope, &["data", "config"]), "foreign").await?;

    Ok(raw_response(()))
}

#[get("/version")]
async fn version_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["data", "config", "flake.lock"]);
    read_file(path).await.map(raw_response)
}

#[post("/update")]
async fn update_endpoint(
    path: web::Path<String>,
    data: web::Json<UpdateData>,
    options: web::Json<CommandOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let data = data.into_inner();
    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Update.to_string());

    spawn(async move { update(&data.inputs, flake(), machine(&container), options).await });

    Ok(json_response(ResponseCommand { id: unit }))
}

#[post("/build")]
async fn build_endpoint(
    path: web::Path<String>,
    options: web::Json<CommandOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Build.to_string());

    spawn(async move { build(flake(), machine(&container), options).await });

    Ok(json_response(ResponseCommand { id: unit }))
}

#[post("/apply")]
async fn apply_endpoint(
    path: web::Path<String>,
    query: web::Query<ApplyQuery>,
    options: web::Json<CommandOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let query = query.into_inner();
    let options = options.into_inner();
    let unit = get_wrapped_unit(&Operation::Apply.to_string());

    let root = get_scoped_path(&scope, &["data"]);

    spawn(async move {
        switch_to_configuration(
            query.when.unwrap_or(ApplyWhen::Now),
            root,
            machine(&container),
            options,
        )
        .await
    });

    Ok(json_response(ResponseCommand { id: unit }))
}
