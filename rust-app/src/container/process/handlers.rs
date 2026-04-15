use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        process::{LogQuery, SystemCtlCommand, execute, logs, status, usage},
        response::{ResponseResult, json_response},
    },
    container::handlers::machine,
};

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

#[get("/{process}/status")]
async fn status_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    status(machine(&container), &process)
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
