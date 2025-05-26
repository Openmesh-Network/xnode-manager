use std::{
    fs::{read_dir, read_to_string, write},
    thread,
};

use actix_identity::Identity;
use actix_web::{
    get,
    web::{self, Path},
    HttpResponse, Responder,
};
use log::warn;
use serde_json::json;

use crate::{
    auth::{models::Scope, utils::has_permission},
    request::models::{CommandInfo, RequestInfo},
    utils::{env::commandstream, error::ResponseError},
};

use super::models::{RequestId, RequestIdResponse, RequestIdResult, RequestsAppData};

#[get("/info/{request_id}")]
async fn request_info(user: Identity, path: Path<RequestId>) -> impl Responder {
    if !has_permission(user, Scope::Request) {
        return HttpResponse::Unauthorized().finish();
    }

    let request_id = path.into_inner();
    let path = commandstream().join(request_id.to_string());
    let commands = read_dir(&path)
        .map(|dir| {
            dir.flat_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.file_type().is_ok_and(|file_type| file_type.is_dir()) {
                        e.file_name().into_string().ok()
                    } else {
                        None
                    }
                })
            })
            .collect()
        })
        .unwrap_or_default();
    let result = {
        let path = path.join("result");
        read_to_string(&path)
            .ok()
            .and_then(|file| serde_json::from_str::<RequestIdResult>(&file).ok())
    };

    HttpResponse::Ok().json(RequestInfo { result, commands })
}

#[get("/info/{request_id}/{command}")]
async fn command_info(user: Identity, path: Path<(RequestId, String)>) -> impl Responder {
    if !has_permission(user, Scope::Request) {
        return HttpResponse::Unauthorized().finish();
    }

    let (request_id, command) = path.into_inner();
    let path = commandstream().join(request_id.to_string()).join(command);

    let command: String;
    let stdout: String;
    let stderr: String;
    {
        let path = path.join("command");
        match read_to_string(&path) {
            Ok(file) => {
                command = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading command from {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }
    {
        let path = path.join("stdout");
        match read_to_string(&path) {
            Ok(file) => {
                stdout = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading stdout from {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }
    {
        let path = path.join("stderr");
        match read_to_string(&path) {
            Ok(file) => {
                stderr = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading stderr from {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }

    let result = {
        let path = path.join("result");
        read_to_string(&path).ok()
    };

    HttpResponse::Ok().json(CommandInfo {
        command,
        stdout,
        stderr,
        result,
    })
}

pub fn return_request_id(
    data: web::Data<RequestsAppData>,
    thread: Box<dyn FnOnce(RequestId) -> RequestIdResult + Send>,
) -> HttpResponse {
    let request_id = match data.request_counter.lock() {
        Ok(mut request_counter) => request_counter.next(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting request id: {}",
                e
            )));
        }
    };

    thread::spawn(move || {
        let result = thread(request_id);
        let path = commandstream().join(request_id.to_string()).join("result");
        if let Err(e) = write(path, json!(result).to_string()) {
            warn!("Could not write result of request {}: {}", request_id, e);
        }
    });

    HttpResponse::Ok().json(RequestIdResponse { request_id })
}
