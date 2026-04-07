use actix_web::{Responder, get, post, web};

use crate::common::{
    process::{LogQuery, SystemCtlCommand, execute, list, logs, usage},
    response::{ResponseResult, json_response},
};

fn machine() -> Option<impl AsRef<str>> {
    None::<String>
}

#[get("/list")]
async fn list_endpoint() -> ResponseResult<impl Responder> {
    list(machine()).await.map(json_response)
}

#[get("/{process}/logs")]
async fn logs_endpoint(
    path: web::Path<String>,
    query: web::Query<LogQuery>,
) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    logs(machine(), &process, &query).await.map(json_response)
}

#[get("/{process}/usage")]
async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    usage(machine(), &process).await.map(json_response)
}

#[post("/{process}/start")]
async fn start_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Start)
        .await
        .map(json_response)
}

#[post("/{process}/stop")]
async fn stop_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Stop)
        .await
        .map(json_response)
}

#[post("/{process}/restart")]
async fn restart_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::Restart)
        .await
        .map(json_response)
}

#[post("/{process}/reload")]
async fn reload_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let process = path.into_inner();
    execute(machine(), &process, SystemCtlCommand::ReloadOrRestart)
        .await
        .map(json_response)
}
