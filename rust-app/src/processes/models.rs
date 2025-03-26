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

#[derive(Serialize, Deserialize, Debug)]
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LogMessage {
    UTF8 { string: String },
    Raw { bytes: Vec<u8> },
}

#[derive(Serialize, Deserialize)]
pub struct Log {
    pub timestamp: u64, // Epoch time in Microseconds
    pub message: LogMessage,
}
