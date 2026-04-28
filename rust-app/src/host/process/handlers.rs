use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        process::{
            LogQuery, ProcessListOptions, SystemCtlCommand, execute, info, list, logs, status,
            usage,
        },
        response::{ResponseResult, json_response, raw_response},
    },
    host::handlers::machine,
};

#[get("/")]
async fn endpoint(options: web::Query<ProcessListOptions>) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    list(machine(), options).await.map(json_response)
}

#[get("/info")]
async fn info_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    info(machine(), &process).await.map(json_response)
}

#[get("/status")]
async fn status_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    status(machine(), &process).await.map(json_response)
}

#[get("/logs")]
async fn logs_endpoint(
    path: web::Path<String>,
    query: web::Query<LogQuery>,
) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    logs(machine(), &process, &query).await.map(json_response)
}

#[get("/usage")]
async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    usage(machine(), &process).await.map(json_response)
}

#[post("/start")]
async fn start_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Start)
        .await
        .map(raw_response)
}

#[post("/stop")]
async fn stop_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Stop)
        .await
        .map(raw_response)
}

#[post("/restart")]
async fn restart_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Restart)
        .await
        .map(raw_response)
}

#[post("/reload")]
async fn reload_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::ReloadOrRestart)
        .await
        .map(raw_response)
}
