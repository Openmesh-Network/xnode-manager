use serde::{Deserialize, Serialize};

use crate::host::permission::models::Permission;

#[derive(Serialize, Deserialize)]
pub struct SetQuery {
    pub detect_changed: Option<bool>,
    pub allow_restart: Option<bool>,
}

pub type SetData = Permission;
