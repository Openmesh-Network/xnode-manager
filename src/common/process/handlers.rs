use futures::future::join_all;
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::systemd,
    process::Info,
    response::{ResponseError, ResponseResult},
    string::escaped_utf8_from_bytes,
};

use super::{
    ProcessOptions, Status,
    models::{
        JournalCtlLog, JournalCtlLogMessage, Log, LogLevel, LogQuery, Process, SystemCtlCommand,
        SystemCtlProcess, Usage,
    },
};

pub async fn list(
    machine: Option<impl AsRef<str>>,
    options: ProcessOptions,
) -> ResponseResult<Vec<Process>> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args([
        "list-units",
        "--type=service",
        "--output=json",
        "--no-pager",
    ]);
    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not retrieve process list of {machine_str}: {e}"
            ))
        })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Process list of {machine_str} could not be decoded as UTF8: {e}."
        ))
    })?;

    let processes = serde_json::from_str::<Vec<SystemCtlProcess>>(&output_str).map_err(|e| {
            ResponseError::new(format!(
                "Process list of {machine_str} could not be parsed to expected format: {e}. Input: {output_str}"
            ))
        })?;

    let mut items = vec![];

    for process in processes {
        let id = process.unit;
        let machine = machine.as_ref();

        let get = async move {
            let mut process_info = None;
            if options.info.unwrap_or(false) {
                process_info = info(machine, &id).await.ok();
            }

            let mut process_status = None;
            if options.status.unwrap_or(false) {
                process_status = status(machine, &id).await.ok();
            }

            let mut process_usage = None;
            if options.usage.unwrap_or(false) {
                process_usage = usage(machine, &id).await.ok();
            }

            Process {
                id,
                info: process_info,
                status: process_status,
                usage: process_usage,
            }
        };

        items.push(get);
    }

    Ok(join_all(items).await)
}

pub async fn info(
    machine: Option<impl AsRef<str>>,
    process: impl AsRef<str>,
) -> ResponseResult<Info> {
    let process = process.as_ref();

    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args(["show", process, "--property=Description"]);
    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not retrieve info of {process} of {machine_str}: {e}"
            ))
        })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Info of {process} of {machine_str} could not be decoded as UTF8: {e}."
        ))
    })?;

    let mut description = None;

    for line in output_str.trim_end().split("\n") {
        if let Some((property, value)) = line.split_once("=") {
            match property {
                "Description" => {
                    description = Some(value.to_string());
                }
                property => {
                    log::warn!(
                        "Info of process {process} of {machine_str} contains unexpected property {property}: {line}"
                    );
                }
            }
        } else {
            log::warn!(
                "Info of process {process} of {machine_str} contains unexpected line: {line}"
            );
        }
    }

    Ok(Info { description })
}

pub async fn status(
    machine: Option<impl AsRef<str>>,
    process: impl AsRef<str>,
) -> ResponseResult<Status> {
    let process = process.as_ref();

    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args(["show", process, "--property=SubState,ExecMainStatus"]);
    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not retrieve status of {process} of {machine_str}: {e}"
            ))
        })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Status of {process} of {machine_str} could not be decoded as UTF8: {e}."
        ))
    })?;

    let mut running = None;
    let mut exit_code = None;

    for line in output_str.trim_end().split("\n") {
        if let Some((property, value)) = line.split_once("=") {
            match property {
                "SubState" => {
                    running = Some(value == "running" || value == "start");
                }
                "ExecMainStatus" => {
                    exit_code = Some(value.parse());
                }
                property => {
                    log::warn!(
                        "Status of process {process} of {machine_str} contains unexpected property {property}: {line}"
                    );
                }
            }
        } else {
            log::warn!(
                "Status of process {process} of {machine_str} contains unexpected line: {line}"
            );
        }
    }

    Ok(Status {
        running: match running {
            Some(running) => running,
            None => {
                return Err(ResponseError::new(format!(
                    "Status of {process} of {machine_str} does not contain running property"
                )));
            }
        },
        exit_code: match exit_code {
            Some(Ok(exit_code)) => exit_code,
            Some(Err(e)) => {
                return Err(ResponseError::new(format!(
                    "Status of {process} of {machine_str} contains invalid exit_code property: {e}"
                )));
            }
            None => {
                return Err(ResponseError::new(format!(
                    "Status of {process} of {machine_str} does not contain exit_code property"
                )));
            }
        },
    })
}

pub async fn logs(
    machine: Option<impl AsRef<str>>,
    process: impl AsRef<str>,
    query: &LogQuery,
) -> ResponseResult<Vec<Log>> {
    let process = process.as_ref();

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

    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
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
        command.args([
            "--since",
            &format!(
                "@{seconds}.{decimal:06}",
                seconds = after / 1_000_000,
                decimal = after % 1_000_000
            ),
        ]);
    }
    if let Some(max) = &query.max {
        command.args(["--lines", &max.to_string()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not retrieve process logs of {process} of {machine_str}: {e}"
            ))
        })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Process logs of {process} of {machine_str} could not be decoded as UTF8: {e}."
        ))
    })?;

    // Add array brackets and , between all entries (separated by newlines)
    let output_json = format!("[{}]", &output_str.trim_end().replace("\n", ","));

    serde_json::from_str::<Vec<JournalCtlLog>>(&output_json)
        .map(|logs| logs
            .into_iter()
            .map(|log| Log {
                timestamp: log.__REALTIME_TIMESTAMP.parse().map(|timestamp: u64| timestamp).unwrap_or(0),
                message: match log.MESSAGE {
                    JournalCtlLogMessage::String(output) => output,
                    JournalCtlLogMessage::Raw(output) => escaped_utf8_from_bytes(output)
                },
                level: journal_ctl_priority_to_log_level(&log.PRIORITY)
            })
            .collect())
        .map_err(|e| {
            ResponseError::new(format!(
                "Process logs of {process} of {machine_str} could not be parsed to expected format: {e}. Input: {output_str}"
            ))
        })
}

pub async fn usage(
    machine: Option<impl AsRef<str>>,
    process: impl AsRef<str>,
) -> ResponseResult<Usage> {
    let process = process.as_ref();

    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args(["show", process, "--property=CPUUsageNSec,MemoryCurrent,IOReadBytes,IOWriteBytes,IPIngressBytes,IPEgressBytes"]);
    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not retrieve usage of {process} of {machine_str}: {e}"
            ))
        })?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Usage of {process} of {machine_str} could not be decoded as UTF8: {e}."
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

    for line in output_str.trim_end().split("\n") {
        if let Some((property, value)) = line.split_once("=") {
            if value == "[not set]" || value == "[no data]" {
                continue;
            }

            let value = match value.parse() {
                Ok(value) => value,
                Err(_) => {
                    log::warn!(
                        "Usage of process {process} of {machine_str} contains unexpected value: {line}"
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
                property => {
                    log::warn!(
                        "Usage of process {process} of {machine_str} contains unexpected property {property}: {line}"
                    );
                }
            }
        } else {
            log::warn!(
                "Usage of process {process} of {machine_str} contains unexpected line: {line}"
            );
        }
    }

    Ok(usage)
}

pub async fn execute(
    machine: Option<impl AsRef<str>>,
    process: impl AsRef<str>,
    systemctl_command: SystemCtlCommand,
) -> ResponseResult<()> {
    let process = process.as_ref();

    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.args([&systemctl_command.to_string(), process]);
    if let Some(machine) = &machine {
        command.args(["--machine", machine.as_ref()]);
    }

    // For error logging
    let machine_str = machine
        .as_ref()
        .map(|m| format!("machine:{m}", m = m.as_ref()))
        .unwrap_or("host".to_string());

    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not perform {systemctl_command} on {process} of {machine_str}: {e}"
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
