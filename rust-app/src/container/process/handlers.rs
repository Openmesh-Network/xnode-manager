use actix_web::{Responder, get, post, web};

use crate::common::{
    process::{LogQuery, SystemCtlCommand, execute, list, logs, usage},
    response::{ResponseResult, json_response},
};

fn machine(container: impl AsRef<str>) -> Option<impl AsRef<str>> {
    let container = container.as_ref();
    Some(format!("{container}.container"))
}

#[get("/list")]
async fn list_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    list(machine(&container)).await.map(json_response)
}

#[get("/{process}/logs")]
async fn logs_endpoint(
    path: web::Path<(String, String)>,
    query: web::Query<LogQuery>,
) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    logs(machine(&container), &process, &query)
        .await
        .map(json_response)
}

#[get("/{process}/usage")]
async fn usage_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    usage(machine(&container), &process)
        .await
        .map(json_response)
}

#[post("/{process}/start")]
async fn start_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Start)
        .await
        .map(json_response)
}

#[post("/{process}/stop")]
async fn stop_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Stop)
        .await
        .map(json_response)
}

#[post("/{process}/restart")]
async fn restart_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Restart)
        .await
        .map(json_response)
}

#[post("/{process}/reload")]
async fn reload_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(
        machine(&container),
        &process,
        SystemCtlCommand::ReloadOrRestart,
    )
    .await
    .map(json_response)
}
