use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Scope {
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Login {
    pub user: String,
}
