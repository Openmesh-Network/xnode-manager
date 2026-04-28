use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProcessListOptions {
    pub status: Option<bool>,
    pub usage: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemCtlProcess {
    pub unit: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<Status>,
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    pub running: bool,
    pub exit_code: u8,
}

#[derive(Serialize, Deserialize)]
pub struct LogQuery {
    pub level: Option<LogLevel>,
    /// Epoch time in microseconds
    pub after: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JournalCtlLogMessage {
    String(String),
    Raw(Vec<u8>),
}
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct JournalCtlLog {
    pub __REALTIME_TIMESTAMP: String,
    pub MESSAGE: JournalCtlLogMessage,
    pub PRIORITY: String,
}

#[derive(Serialize, Deserialize)]
pub struct Log {
    /// Epoch time in microseconds
    pub timestamp: u64,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub cpu: Option<u64>,
    pub memory: Option<u64>,
    pub network_ingress: Option<u64>,
    pub network_egress: Option<u64>,
    pub disk_read: Option<u64>,
    pub disk_write: Option<u64>,
}

pub enum SystemCtlCommand {
    Start,
    Stop,
    Restart,
    ReloadOrRestart,
}
impl Display for SystemCtlCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SystemCtlCommand::Start => "start",
                SystemCtlCommand::Stop => "stop",
                SystemCtlCommand::Restart => "restart",
                SystemCtlCommand::ReloadOrRestart => "reload-or-restart",
            }
        )
    }
}
