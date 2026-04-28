use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
};

use actix_web::{Responder, get, web};
use tokio::process::Command;

use crate::{
    common::{
        command::execute_command_simple,
        env::systemd,
        file::{ReadFolderOptions, read_file, read_folder},
        response::{ResponseError, ResponseResult, json_response},
        string::escaped_utf8_from_bytes,
    },
    host::hardware::network::models::{Address, Network, NetworkCtlStatus},
};

use super::models::{Info, Usage};

#[get("/")]
async fn endpoint() -> ResponseResult<impl Responder> {
    let path = "/sys/class/net";
    read_folder(path, &ReadFolderOptions { metadata: None })
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| Network { id: item.name })
                .collect::<Vec<Network>>()
        })
        .map(json_response)
}

#[get("/info")]
async fn info_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let interface = path.into_inner();

    let mut command = Command::new(format!("{}networkctl", systemd()));
    command.args(["status", &interface, "--json", "short"]);

    let output = execute_command_simple(command)
        .await
        .map_err(|e| ResponseError::new(format!("Could not get info of {interface}: {e}")))?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Info of {interface} could not be decoded as UTF8: {e}."
        ))
    })?;

    let info = serde_json::from_str::<NetworkCtlStatus>(&output_str).map_err(|e| {
        ResponseError::new(format!(
            "Info of {interface} could not be parsed to expected format: {e}. Input: {output_str}"
        ))
    })?;

    Ok(json_response(Info {
        mac: info
            .HardwareAddress
            .into_iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(":"),
        addresses: info
            .Addresses
            .iter()
            .filter_map(|address| match address.Family {
                2 => {
                    if let Ok(octets) = address.HardwareAddress.clone().try_into() {
                        Some(Address::IPv4(Ipv4Addr::from_octets(octets).to_string()))
                    } else {
                        log::warn!(
                            "Could not parse ipv4 address {address:?}",
                            address = address.HardwareAddress
                        );
                        None
                    }
                }
                10 => {
                    if let Ok(octets) = address.HardwareAddress.clone().try_into() {
                        Some(Address::IPv4(Ipv6Addr::from_octets(octets).to_string()))
                    } else {
                        log::warn!(
                            "Could not parse ipv6 address {address:?}",
                            address = address.HardwareAddress
                        );
                        None
                    }
                }
                _ => None,
            })
            .collect(),
    }))
}

#[get("/usage")]
async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let interface = path.into_inner();

    let path = Path::new("/sys/class/net")
        .join(&interface)
        .join("statistics");

    let rx = read_file(path.join("rx_bytes"))
        .await
        .map(escaped_utf8_from_bytes)?
        .trim()
        .parse::<u64>()
        .map_err(|e| {
            ResponseError::new(format!("Could not parse rx_bytes for {interface}: {e}"))
        })?;

    let tx = read_file(path.join("tx_bytes"))
        .await
        .map(escaped_utf8_from_bytes)?
        .trim()
        .parse::<u64>()
        .map_err(|e| {
            ResponseError::new(format!("Could not parse tx_bytes for {interface}: {e}"))
        })?;

    Ok(json_response(Usage {
        received: rx,
        transmitted: tx,
    }))
}
