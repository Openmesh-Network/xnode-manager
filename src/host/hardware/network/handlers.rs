use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
};

use actix_web::{Responder, get, web};
use futures::future::join_all;
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::systemd,
    file::{ReadFolderOptions, read_file, read_folder},
    response::{ResponseError, ResponseResult, json_response},
    string::escaped_utf8_from_bytes,
};

use super::models::{Address, Info, Network, NetworkCtlStatus, NetworkOptions, Usage};

#[get("")]
async fn memory_endpoint(options: web::Query<NetworkOptions>) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    let path = "/sys/class/net";
    let networks = read_folder(path, &ReadFolderOptions::default())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| item.name)
                .collect::<Vec<String>>()
        })?;

    let mut items = vec![];

    for network in networks {
        let get = async move {
            let mut network_info = None;
            if options.info.unwrap_or(false) {
                network_info = info(&network).await.ok();
            }

            let mut network_usage = None;
            if options.usage.unwrap_or(false) {
                network_usage = usage(&network).await.ok();
            }

            Network {
                id: network,
                info: network_info,
                usage: network_usage,
            }
        };

        items.push(get);
    }

    Ok(json_response(join_all(items).await))
}

#[get("/info")]
async fn info_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let network = path.into_inner();
    info(&network).await.map(json_response)
}

#[get("/usage")]
async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let network = path.into_inner();
    usage(&network).await.map(json_response)
}

async fn info(network: impl AsRef<str>) -> ResponseResult<Info> {
    let network = network.as_ref();

    let mut command = Command::new(format!("{}networkctl", systemd()));
    command.args(["status", network, "--json", "short"]);

    let output = execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map_err(|e| ResponseError::new(format!("Could not get info of {network}: {e}")))?;
    let output_str = String::from_utf8(output).map_err(|e| {
        ResponseError::new(format!(
            "Info of {network} could not be decoded as UTF8: {e}."
        ))
    })?;

    let info = serde_json::from_str::<NetworkCtlStatus>(&output_str).map_err(|e| {
        ResponseError::new(format!(
            "Info of {network} could not be parsed to expected format: {e}. Input: {output_str}"
        ))
    })?;

    Ok(Info {
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
    })
}

async fn usage(network: impl AsRef<str>) -> ResponseResult<Usage> {
    let network = network.as_ref();

    let path = Path::new("/sys/class/net").join(network).join("statistics");

    let rx = read_file(path.join("rx_bytes"))
        .await
        .map(escaped_utf8_from_bytes)?
        .trim()
        .parse::<u64>()
        .map_err(|e| ResponseError::new(format!("Could not parse rx_bytes for {network}: {e}")))?;

    let tx = read_file(path.join("tx_bytes"))
        .await
        .map(escaped_utf8_from_bytes)?
        .trim()
        .parse::<u64>()
        .map_err(|e| ResponseError::new(format!("Could not parse tx_bytes for {network}: {e}")))?;

    Ok(Usage {
        received: rx,
        transmitted: tx,
    })
}
