use std::process::Command;

use actix_identity::Identity;
use actix_web::{get, web::Path, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    processes::models::LogMessage,
    utils::{
        command::{execute_command, CommandOutput, CommandOutputError},
        env::systemd,
        error::ResponseError,
    },
};

use super::models::{JournalCtlLog, JournalCtlLogMessage, Log, Process, SystemCtlProcess};

#[get("/list/{container}")]
async fn list(user: Identity, path: Path<String>) -> impl Responder {
    if !has_permission(user, Scope::Processes) {
        return HttpResponse::Unauthorized().finish();
    }

    let container_id = path.into_inner();
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command
        .arg("list-units")
        .arg("--machine")
        .arg(&container_id)
        .arg("--type=service")
        .arg("--output=json")
        .arg("--no-pager");
    match execute_command(command) {
        Ok(output) => match output {
            CommandOutput::Output(output_str) => {
                match serde_json::from_str::<Vec<SystemCtlProcess>>(&output_str) {
                    Ok(output_parsed) => {
                        let response: Vec<Process> = output_parsed
                            .into_iter()
                            .map(|process| Process {
                                name: process.unit,
                                description: Some(process.description),
                                running: process.sub == "running",
                            })
                            .collect();
                        HttpResponse::Ok().json(response)
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                            "Logs could not be parsed to expected format: {}. Logs: {}",
                            e, output_str
                        )))
                    }
                }
            }
            CommandOutput::OutputRaw(output, e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Logs could not be decoded as UTF8: {}. Logs: {:?}.",
                    e, output
                )))
            }
        },
        Err(e) => match e {
            CommandOutputError::OutputErrorRaw(output, e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get units of container {} command: Output could not be decoded: {}. Output: {:?}",
                    &container_id,
                    e,
                    output,
                )))
            }
            CommandOutputError::OutputError(output) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get units of container {} command: {}",
                    &container_id, output,
                )))
            }
            CommandOutputError::CommandError(e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get units of container {} command: {}",
                    &container_id, e
                )))
            }
        },
    }
}

#[get("/logs/{container}/{process}")]
async fn logs(user: Identity, path: Path<(String, String)>) -> impl Responder {
    if !has_permission(user, Scope::Processes) {
        return HttpResponse::Unauthorized().finish();
    }

    let (container_id, process) = path.into_inner();
    let mut command = Command::new(format!("{}journalctl", systemd()));
    command
        .arg("--machine")
        .arg(&container_id)
        .arg("--unit")
        .arg(&process)
        .arg("--output=json")
        .arg("--no-pager")
        .arg("--output-fields")
        .arg("__REALTIME_TIMESTAMP,MESSAGE");
    match execute_command(command) {
        Ok(output) => match output {
            CommandOutput::Output(output_str) => {
                let output_json = format!(
                    "[{}]",
                    &output_str[..output_str.len() - 1].replace("\n", ",")
                ); // Add array brackets and , between all entries (separated by newlines)
                match serde_json::from_str::<Vec<JournalCtlLog>>(&output_json) {
                    Ok(output_parsed) => {
                        let response: Vec<Log> = output_parsed
                            .into_iter()
                            .map(|log| Log {
                                timestamp: log.__REALTIME_TIMESTAMP.parse().unwrap_or(0),
                                message: match log.MESSAGE {
                                    JournalCtlLogMessage::String(string) => LogMessage::UTF8 { string },
                                    JournalCtlLogMessage::Raw(bytes) => LogMessage::Raw { bytes }
                                },
                            })
                            .collect();
                        HttpResponse::Ok().json(response)
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                            "Logs of process {} of container {} could not be parsed to expected format: {}. Logs: {}",
                            &process, &container_id, e, output_json
                        )))
                    }
                }
            }
            CommandOutput::OutputRaw(output, e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Logs of process {} of container {} could not be decoded as UTF8: {}. Logs: {:?}.",
                &process, &container_id, e, output
            )))
            }
        },
        Err(e) => match e {
            CommandOutputError::OutputErrorRaw(output, e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get logs of process {} of container {} command: Output could not be decoded: {}. Output: {:?}",
                    &process, &container_id,
                    e,
                    output,
                )))
            }
            CommandOutputError::OutputError(output) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get logs of process {} of container {} command: {}",
                    &process, &container_id, output,
                )))
            }
            CommandOutputError::CommandError(e) => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error executing get logs of process {} of container {} command: {}",
                    &process, &container_id, e
                )))
            }
        },
    }
}
