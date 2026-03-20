use tokio::process::Command;

use crate::common::{
    command::execute_command_simple, env::systemd, error::ResponseError,
    string::escaped_utf8_from_bytes,
};

use super::models::{
    JournalCtlLog, JournalCtlLogMessage, Log, LogLevel, LogQuery, Process, SystemCtlCommand,
    SystemCtlProcess, Usage,
};

pub async fn list(machine: Option<&str>) -> Result<Vec<Process>, ResponseError> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args([
        "list-units",
        "--type=service",
        "--output=json",
        "--no-pager",
    ]);
    if let Some(machine) = machine {
        command.args(["--machine", machine]);
    }

    // For error logging
    let machine = machine
        .map(|m| format!("machine:{m}"))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command).await.map_err(|e| {
        ResponseError::new(format!("Could not retrieve process list of {machine}: {e}"))
    })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Process list of {machine} could not be decoded as UTF8: {e}."
        ))
    })?;

    serde_json::from_str::<Vec<SystemCtlProcess>>(&output_str)
        .map(|processes| processes
            .into_iter()
            .map(|process| Process {
                name: process.unit,
                description: Some(process.description),
                running: process.sub == "running",
            })
            .collect())
        .map_err(|e| {
            ResponseError::new(format!(
                "Process list of {machine} could not be parsed to expected format: {e}. Input: {output_str}"
            ))
        })
}

pub async fn logs(
    machine: Option<&str>,
    process: &str,
    query: &LogQuery,
) -> Result<Vec<Log>, ResponseError> {
    let mut command = Command::new(format!("{}journalctl", systemd()));
    command.args([
        "--unit",
        process,
        "--output=json",
        "--all",
        "--no-pager",
        "--output-field",
        "__REALTIME_TIMESTAMP,MESSAGE,PRIORITY",
    ]);

    if let Some(machine) = machine {
        command.args(["--machine", machine]);
    }
    if let Some(level) = &query.level {
        command.args([
            "--priority",
            &match level {
                LogLevel::Error => 3,
                LogLevel::Warn => 4,
                LogLevel::Info => 7,
                LogLevel::Unknown => 7,
            }
            .to_string(),
        ]);
    }
    if let Some(after) = &query.after {
        command.args(["--since", &format!("@{after}")]);
    }
    if let Some(max) = &query.max {
        command.args(["--lines", &max.to_string()]);
    }

    // For error logging
    let machine = machine
        .map(|m| format!("machine:{m}"))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command).await.map_err(|e| {
        ResponseError::new(format!(
            "Could not retrieve process logs of {process} of {machine}: {e}"
        ))
    })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Process logs of {process} of {machine} could not be decoded as UTF8: {e}."
        ))
    })?;

    // Add array brackets and , between all entries (separated by newlines)
    let output_json = format!(
        "[{}]",
        &output_str[..output_str.len() - 1].replace("\n", ",")
    );

    serde_json::from_str::<Vec<JournalCtlLog>>(&output_json)
        .map(|logs| logs
            .into_iter()
            .map(|log| Log {
                timestamp: log.__REALTIME_TIMESTAMP.parse().unwrap_or(0),
                message: match log.MESSAGE {
                    JournalCtlLogMessage::String(output) => output,
                    JournalCtlLogMessage::Raw(output) => escaped_utf8_from_bytes(output)
                },
                level: journal_ctl_priority_to_log_level(&log.PRIORITY)
            })
            .collect())
        .map_err(|e| {
            ResponseError::new(format!(
                "Process logs of {process} of {machine} could not be parsed to expected format: {e}. Input: {output_str}"
            ))
        })
}

pub async fn usage(machine: Option<&str>, process: &str) -> Result<Usage, ResponseError> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args(["show", process, "--property=CPUUsageNSec,MemoryCurrent,IOReadBytes,IOWriteBytes,IPIngressBytes,IPEgressBytes"]);
    if let Some(machine) = machine {
        command.args(["--machine", machine]);
    }

    // For error logging
    let machine = machine
        .map(|m| format!("machine:{m}"))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command).await.map_err(|e| {
        ResponseError::new(format!(
            "Could not retrieve usage of {process} of {machine}: {e}"
        ))
    })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Usage of {process} of {machine} could not be decoded as UTF8: {e}."
        ))
    })?;

    let mut usage = Usage {
        cpu: None,
        memory: None,
        network_ingress: None,
        network_egress: None,
        disk_read: None,
        disk_write: None,
    };

    for line in output_str.split("\n") {
        if let Some((property, value)) = line.split_once("\n") {
            if value == "[not set]" || value == "[no data]" {
                continue;
            }

            let value = match value.parse() {
                Ok(value) => value,
                Err(_) => {
                    log::warn!(
                        "Usage of process {process} of {machine} contains unexpected value: {line}"
                    );
                    continue;
                }
            };

            match property {
                "CPUUsageNSec" => {
                    usage.cpu = Some(value);
                }
                "MemoryCurrent" => {
                    usage.memory = Some(value);
                }
                "IPIngressBytes" => {
                    usage.network_ingress = Some(value);
                }
                "IPEgressBytes" => {
                    usage.network_egress = Some(value);
                }
                "IOReadBytes" => {
                    usage.disk_read = Some(value);
                }
                "IOWriteBytes" => {
                    usage.disk_write = Some(value);
                }
                _ => {
                    log::warn!(
                        "Usage of process {process} of {machine} contains unexpected property: {line}"
                    );
                }
            }
        } else {
            log::warn!("Usage of process {process} of {machine} contains unexpected line: {line}");
        }
    }

    Ok(usage)
}

pub async fn execute(
    machine: Option<&str>,
    process: &str,
    systemctl_command: SystemCtlCommand,
) -> Result<(), ResponseError> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args([&systemctl_command.to_string(), process]);
    if let Some(machine) = machine {
        command.args(["--machine", machine]);
    }

    // For error logging
    let machine = machine
        .map(|m| format!("machine:{m}"))
        .unwrap_or("host".to_string());

    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not perform {systemctl_command} on {process} of {machine}: {e}"
            ))
        })
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
