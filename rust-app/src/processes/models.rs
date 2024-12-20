use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SystemCtlProcess {
    pub unit: String,
    pub active: String,
}

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct JournalCtlLog {
    pub __REALTIME_TIMESTAMP: String,
    pub MESSAGE: String,
}

#[derive(Serialize, Deserialize)]
pub struct Log {
    pub timestamp: u64, // Epoch time in Microseconds
    pub message: String,
}
