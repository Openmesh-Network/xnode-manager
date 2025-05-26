use serde::{Deserialize, Serialize};

pub type RequestId = u32;

#[derive(Serialize, Deserialize)]
pub struct RequestIdResponse {
    pub request_id: RequestId,
}

#[derive(Serialize, Deserialize)]
pub enum RequestIdResult {
    Success { body: Option<String> },
    Error { error: String },
}

#[derive(Serialize, Deserialize)]
pub struct RequestInfo {
    pub commands: Vec<String>,
    pub result: Option<RequestIdResult>,
}

#[derive(Serialize, Deserialize)]
pub struct CommandInfo {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub result: Option<String>,
}
