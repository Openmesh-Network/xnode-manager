use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkOptions {
    pub usage: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Network {
    pub id: String,
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize)]
pub enum Address {
    IPv4(String),
    IPv6(String),
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkCtlAddress {
    pub Family: u8,
    pub HardwareAddress: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkCtlStatus {
    pub HardwareAddress: Vec<u8>,
    pub Addresses: Vec<NetworkCtlAddress>,
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub mac: String,
    pub addresses: Vec<Address>,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub received: u64,
    pub transmitted: u64,
}
