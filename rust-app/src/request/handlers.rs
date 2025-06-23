use std::{
    fs::{create_dir_all, read, read_dir, read_to_string, write},
    thread,
};

use actix_web::{get, web, HttpResponse, Responder};
use log::warn;
use serde_json::json;

use crate::{
    request::models::{CommandInfo, RequestInfo},
    utils::{env::commandstream, error::ResponseError, output::Output},
};

use super::models::{RequestId, RequestIdResponse, RequestIdResult};

#[get("/info/{request_id}")]
async fn request_info(path: web::Path<RequestId>) -> impl Responder {
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
async fn command_info(path: web::Path<(RequestId, String)>) -> impl Responder {
    let (request_id, command) = path.into_inner();
    let path = commandstream().join(request_id.to_string()).join(command);

    let command: String;
    let stdout: Output;
    let stderr: Output;
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
        match read(&path) {
            Ok(file) => {
                stdout = file.into();
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
        match read(&path) {
            Ok(file) => {
                stderr = file.into();
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
    thread: Box<dyn FnOnce(RequestId) -> RequestIdResult + Send>,
) -> HttpResponse {
    let request_id = get_request_id();

    thread::spawn(move || {
        let path = commandstream().join(request_id.to_string());
        if let Err(e) = create_dir_all(&path) {
            warn!(
                "Could not create directory for request {} at {}: {}",
                request_id,
                path.display(),
                e
            );
        }
        let result = thread(request_id);
        {
            let path = path.join("result");
            if let Err(e) = write(&path, json!(result).to_string()) {
                warn!(
                    "Could not write result of request {} to {}: {}",
                    request_id,
                    path.display(),
                    e
                );
            }
        }
    });

    HttpResponse::Ok().json(RequestIdResponse { request_id })
}

fn get_request_id() -> RequestId {
    read_dir(commandstream())
        .map(|dir| {
            dir.map(|entry| {
                entry
                    .map(|e| {
                        e.file_name()
                            .into_string()
                            .map(|s| s.parse::<u32>().unwrap_or(0))
                            .unwrap_or(0)
                    })
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
        })
        .unwrap_or(0)
        + 1
}
