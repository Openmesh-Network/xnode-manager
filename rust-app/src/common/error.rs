use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseError {
    pub error: String,
}

impl ResponseError {
    pub fn new(error: impl Display) -> Self {
        let error = error.to_string();
        log::warn!("Response error: {error}");

        Self { error }
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
