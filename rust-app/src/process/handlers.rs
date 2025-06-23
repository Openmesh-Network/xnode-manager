use std::process::Command;

use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{
    process::models::{LogQuery, ProcessCommand},
    request::{handlers::return_request_id, models::RequestIdResult},
    utils::{
        command::{execute_command, CommandExecutionMode},
        env::systemd,
        error::ResponseError,
        output::Output,
    },
};

use super::models::{
    JournalCtlLog, JournalCtlLogMessage, Log, LogLevel, Process, SystemCtlProcess,
};

#[get("/list/{scope}")]
async fn list(path: web::Path<String>) -> impl Responder {
    let scope = path.into_inner();
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command
        .arg("list-units")
        .arg("--type=service")
        .arg("--output=json")
        .arg("--no-pager");
    if scope.starts_with("container:") {
        command
            .arg("--machine")
            .arg(scope.replace("container:", ""));
    }
    match execute_command(command, CommandExecutionMode::Simple) {
        Ok(output) => match output.into() {
            Output::UTF8 { output: output_str } => {
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
            Output::Bytes { output } => HttpResponse::InternalServerError().json(
                ResponseError::new(format!("Logs could not be decoded as UTF8: {:?}.", output)),
            ),
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error executing get units of {} command: {}",
            &scope, e
        ))),
    }
}

#[get("/logs/{scope}/{process}")]
async fn logs(path: web::Path<(String, String)>, query: web::Query<LogQuery>) -> impl Responder {
    let (scope, process) = path.into_inner();
    let max_logs = query.max.unwrap_or(100);
    let log_level = &query.level;

    let mut command = Command::new(format!("{}journalctl", systemd()));
    command
        .arg("--unit")
        .arg(&process)
        .arg("--output=json")
        .arg("--all") // Prevents messages > 4096 bytes to be encoded as null
        .arg("--no-pager")
        .arg("--output-fields")
        .arg("__REALTIME_TIMESTAMP,MESSAGE,PRIORITY")
        .arg("--lines")
        .arg(max_logs.to_string());
    if scope.starts_with("container:") {
        command
            .arg("--machine")
            .arg(scope.replace("container:", ""));
    }
    if let Some(level) = log_level {
        command.arg("--priority").arg(
            match level {
                LogLevel::Error => 3,
                LogLevel::Warn => 4,
                LogLevel::Info => 7,
                LogLevel::Unknown => 7,
            }
            .to_string(),
        );
    }
    match execute_command(command, CommandExecutionMode::Simple) {
        Ok(output) => match output.into() {
            Output::UTF8 { output: output_str } => {
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
                                    JournalCtlLogMessage::String(output) => Output::UTF8 { output },
                                    JournalCtlLogMessage::Raw(output) => Output::Bytes { output }
                                },
                                level: journal_ctl_priority_to_log_level(&log.PRIORITY)
                            })
                            .collect();
                        HttpResponse::Ok().json(response)
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                            "Logs of process {} of {} could not be parsed to expected format: {}. Logs: {}",
                            &process, &scope, e, output_json
                        )))
                    }
                }
            }
            Output::Bytes { output } => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Logs of process {} of {} could not be decoded as UTF8: {:?}.",
                    &process, &scope, output
                )))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error executing get logs of process {} of {} command: {}",
            &process, &scope, e
        ))),
    }
}

#[post("/execute/{scope}/{process}")]
async fn execute(
    path: web::Path<(String, String)>,
    command: web::Json<ProcessCommand>,
) -> impl Responder {
    return_request_id(Box::new(move |request_id| {
        let (scope, process) = path.into_inner();
        let systemd_command = match command.into_inner() {
            ProcessCommand::Start => "start",
            ProcessCommand::Stop => "stop",
            ProcessCommand::Restart => "restart",
        };

        let mut command = Command::new(format!("{}systemctl", systemd()));
        command.arg(systemd_command).arg(&process);
        if scope.starts_with("container:") {
            command
                .arg("--machine")
                .arg(scope.replace("container:", ""));
        }

        match execute_command(command, CommandExecutionMode::Stream { request_id }) {
            Ok(output) => RequestIdResult::Success {
                body: match output.into() {
                    Output::UTF8 { output } => Some(output),
                    _ => None,
                },
            },
            Err(e) => RequestIdResult::Error {
                error: format!(
                    "Erroring executing {} on {} of {}: {}",
                    systemd_command, process, scope, e
                ),
            },
        }
    }))
}

fn journal_ctl_priority_to_log_level(priority: &str) -> LogLevel {
    let priority_num = match str::parse::<u8>(priority) {
        Ok(num) => num,
        Err(_) => {
            return LogLevel::Unknown;
        }
    };

    if priority_num <= 3 {
        return LogLevel::Error;
    }
    if priority_num <= 4 {
        return LogLevel::Warn;
    }
    if priority_num <= 7 {
        return LogLevel::Info;
    }

    LogLevel::Unknown
}
