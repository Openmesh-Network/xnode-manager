use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseError {
    pub error: String,
}

impl ResponseError {
    pub fn new(error: impl Display) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}
