use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: u32,
    pub group: u32,
    pub description: String,
    pub home: String,
    pub login: String,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub id: u32,
    pub members: Vec<String>,
}
