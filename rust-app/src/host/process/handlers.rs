use actix_web::{Responder, get, post, web};

use crate::common::{
    process::{LogQuery, SystemCtlCommand, execute, list, logs, usage},
    response::wrap_json_response,
};

#[get("/list")]
async fn list_endpoint() -> impl Responder {
    wrap_json_response(list(None).await)
}

#[get("/{process}/logs")]
async fn logs_endpoint(path: web::Path<String>, query: web::Query<LogQuery>) -> impl Responder {
    let process = path.into_inner();
    wrap_json_response(logs(None, &process, &query).await)
}

#[get("/{process}/usage")]
async fn usage_endpoint(path: web::Path<String>) -> impl Responder {
    let process = path.into_inner();
    wrap_json_response(usage(None, &process).await)
}

#[post("/{process}/start")]
async fn start_endpoint(path: web::Path<String>) -> impl Responder {
    let process = path.into_inner();
    wrap_json_response(execute(None, &process, SystemCtlCommand::Start).await)
}

#[post("/{process}/stop")]
async fn stop_endpoint(path: web::Path<String>) -> impl Responder {
    let process = path.into_inner();
    wrap_json_response(execute(None, &process, SystemCtlCommand::Stop).await)
}

#[post("/{process}/restart")]
async fn restart_endpoint(path: web::Path<String>) -> impl Responder {
    let process = path.into_inner();
    wrap_json_response(execute(None, &process, SystemCtlCommand::ReloadOrRestart).await)
}
