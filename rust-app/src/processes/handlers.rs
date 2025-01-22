use std::process::Command;

use actix_identity::Identity;
use actix_web::{get, web::Path, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    utils::error::ResponseError,
};

use super::models::{JournalCtlLog, Log, Process, SystemCtlProcess};

#[get("/list")]
async fn list(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Processes) {
        return HttpResponse::Unauthorized().finish();
    }

    let systemctl = Command::new("systemctl")
        .arg("list-units")
        .arg("--type=service")
        .arg("--state=running")
        .arg("-o")
        .arg("json")
        .arg("--no-pager")
        .output();
    match systemctl {
        Ok(output_raw) => match String::from_utf8(output_raw.stdout) {
            Ok(output_str) => match serde_json::from_str::<Vec<SystemCtlProcess>>(&output_str) {
                Ok(output_parsed) => {
                    let response: Vec<Process> = output_parsed
                        .iter()
                        .map(|process| Process {
                            name: process.unit.clone(),
                            active: process.active == "active",
                        })
                        .collect();
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Logs could not be parsed to expected format: {}. Logs: {}",
                    e, output_str
                ))),
            },
            Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Logs could not be decoded as UTF8: {}.",
                e
            ))),
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Command execution failed: {}.",
            e
        ))),
    }
}

#[get("/logs/{process}")]
async fn logs(user: Identity, path: Path<String>) -> impl Responder {
    if !has_permission(user, Scope::Processes) {
        return HttpResponse::Unauthorized().finish();
    }

    let process = path.into_inner();
    let journalctl = Command::new("journalctl")
        .arg("-u")
        .arg(process)
        .arg("-o")
        .arg("json")
        .arg("--no-pager")
        .arg("--output-fields")
        .arg("__REALTIME_TIMESTAMP,MESSAGE")
        .output();
    match journalctl {
        Ok(output_raw) => match String::from_utf8(output_raw.stdout) {
            Ok(output_str) => {
                let output_json = format!(
                    "[{}]",
                    &output_str[..output_str.len() - 1].replace("\n", ",")
                ); // Add array brackets and , between all entries (separated by newlines)
                match serde_json::from_str::<Vec<JournalCtlLog>>(&output_json) {
                    Ok(output_parsed) => {
                        let response: Vec<Log> = output_parsed
                            .iter()
                            .map(|log| Log {
                                timestamp: match log.__REALTIME_TIMESTAMP.parse() {
                                    Ok(num) => num,
                                    Err(_) => 0,
                                },
                                message: log.MESSAGE.clone(),
                            })
                            .collect();
                        HttpResponse::Ok().json(response)
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                            "Logs could not be parsed to expected format: {}. Logs: {}",
                            e, output_json
                        )))
                    }
                }
            }
            Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Logs could not be decoded as UTF8: {}.",
                e
            ))),
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Command execution failed: {}.",
            e
        ))),
    }
}
