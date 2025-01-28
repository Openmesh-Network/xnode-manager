use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Scope {
    Processes,
    ResourceUsage,
    OS,
    Config,
}

#[derive(Serialize, Deserialize)]
pub enum AdditionalVerification {
    WalletSignature { v: u8, r: [u8; 32], s: [u8; 32] },
}

#[derive(Serialize, Deserialize)]
pub enum LoginMethod {
    WalletSignature { v: u8, r: [u8; 32], s: [u8; 32] },
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub login_method: LoginMethod,
}
