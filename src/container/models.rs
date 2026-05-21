use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Container {
    pub id: String,
}
