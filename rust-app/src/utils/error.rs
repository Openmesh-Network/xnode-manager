use std::fmt::Display;

use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseError {
    pub error: String,
}

impl ResponseError {
    pub fn new(error: impl Display) -> Self {
        warn!("Response error: {}", error);
        Self {
            error: error.to_string(),
        }
    }
}
