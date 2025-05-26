use std::{
    fs::read_dir,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::utils::env::commandstream;

pub type RequestId = u32;

pub struct RequestCounter {
    pub current: RequestId,
}
impl RequestCounter {
    pub fn new(start: RequestId) -> Self {
        Self { current: start }
    }

    pub fn next(&mut self) -> RequestId {
        self.current += 1;
        self.current
    }
}

pub struct RequestsAppData {
    pub request_counter: Arc<Mutex<RequestCounter>>,
}

impl Default for RequestsAppData {
    fn default() -> Self {
        RequestsAppData {
            request_counter: Arc::new(Mutex::new(RequestCounter::new(
                read_dir(commandstream())
                    .map(|dir| {
                        dir.map(|entry| {
                            entry
                                .map(|e| {
                                    e.file_name()
                                        .into_string()
                                        .map(|s| s.parse::<u32>().unwrap_or(0))
                                        .unwrap_or(0)
                                })
                                .unwrap_or(0)
                        })
                        .max()
                        .unwrap_or(0)
                    })
                    .unwrap_or(0),
            ))),
        }
    }
}

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
