use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseError {
    pub error: String,
}

impl ResponseError {
    pub fn new(error: impl Display) -> Self {
        log::warn!("Response error: {}", error);
        Self {
            error: error.to_string(),
        }
    }
}
