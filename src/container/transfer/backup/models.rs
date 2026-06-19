use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Compression {
    Zstd,
}

#[derive(Serialize, Deserialize)]
pub struct ReceiveQuery {
    pub compression: Option<Compression>,
}

#[derive(Serialize, Deserialize)]
pub struct SendQuery {
    pub compression: Option<Compression>,
    pub compression_level: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub send: Vec<String>,
    pub receive: String,
    pub common: Option<Vec<String>>,
}
