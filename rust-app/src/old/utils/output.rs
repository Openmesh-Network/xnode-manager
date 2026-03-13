use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Output {
    Bytes { output: Vec<u8> },
    UTF8 { output: String },
}

impl From<Vec<u8>> for Output {
    fn from(value: Vec<u8>) -> Self {
        match String::from_utf8(value.clone()) {
            Ok(output) => Output::UTF8 { output },
            Err(_) => Output::Bytes { output: value },
        }
    }
}
