use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        process::{
            LogQuery, ProcessOptions, SystemCtlCommand, execute, info, list, logs, status, usage,
        },
        response::{ResponseResult, json_response},
    },
    container::handlers::machine,
};

#[get("")]
async fn process_endpoint(
    path: web::Path<String>,
    options: web::Query<ProcessOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let options = options.into_inner();

    list(machine(container), options).await.map(json_response)
}

#[get("/info")]
async fn info_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    info(machine(&container), &process).await.map(json_response)
}

#[get("/status")]
async fn status_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    status(machine(&container), &process)
        .await
        .map(json_response)
}

#[get("/logs")]
async fn logs_endpoint(
    path: web::Path<(String, String)>,
    query: web::Query<LogQuery>,
) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    logs(machine(&container), &process, &query)
        .await
        .map(json_response)
}

#[get("/usage")]
async fn usage_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    usage(machine(&container), &process)
        .await
        .map(json_response)
}

#[post("/start")]
async fn start_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Start)
        .await
        .map(json_response)
}

#[post("/stop")]
async fn stop_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Stop)
        .await
        .map(json_response)
}

#[post("/restart")]
async fn restart_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, process) = path.into_inner();
    execute(machine(&container), &process, SystemCtlCommand::Restart)
        .await
        .map(json_response)
}

#[post("/reload")]
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
