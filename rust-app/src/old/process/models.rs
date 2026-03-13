use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SystemCtlProcess {
    pub unit: String,
    pub description: String,
    pub sub: String,
}

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub description: Option<String>,
    pub running: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LogQuery {
    pub max: Option<u32>,
    pub level: Option<LogLevel>,
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
    pub timestamp: u64,  // Epoch time in Microseconds
    pub message: String, // base64 bytes
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

#[derive(Serialize, Deserialize)]
pub enum ProcessCommand {
    Start,
    Stop,
    Restart,
}
