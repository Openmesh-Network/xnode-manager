use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Scope {
    Processes,
    ResourceUsage,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LoginMethod {
    WalletSignature { v: u8, r: [u8; 32], s: [u8; 32] },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    pub login_method: LoginMethod,
}
