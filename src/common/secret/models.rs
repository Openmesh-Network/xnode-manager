use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SecretOptions {
    pub get: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Secret {
    pub id: String,
    pub get: Option<Vec<u8>>,
}
