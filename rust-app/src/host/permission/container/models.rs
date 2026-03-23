use serde::{Deserialize, Serialize};

use crate::host::permission::models::Permission;

#[derive(Serialize, Deserialize)]
pub struct SetQuery {
    pub allow_restart: bool,
}

pub type SetData = Permission;
