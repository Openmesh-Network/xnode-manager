use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::host::permission::models::Permission;

fn env_var(id: &str) -> Option<String> {
    std::env::var(id)
        .inspect_err(|e| {
            log::warn!("Could not read env var {}: {}", id, e);
        })
        .ok()
}

pub fn datadir() -> PathBuf {
    env_var("DATADIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/xnode-manager").to_path_buf())
}

pub fn socket() -> PathBuf {
    env_var("SOCKET")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/run/xnode-manager/.socket").to_path_buf())
}

pub fn nix() -> String {
    env_var("NIX").unwrap_or("".to_string())
}

pub fn systemd() -> String {
    env_var("SYSTEMD").unwrap_or("".to_string())
}

pub fn btrfs() -> String {
    env_var("BTRFS").unwrap_or("".to_string())
}

#[derive(Serialize, Deserialize)]
pub struct DefaultPermission {
    pub container: Permission,
    #[serde(rename = "virtual-machine")]
    pub virtual_machine: Permission,
}
pub fn default_permission() -> DefaultPermission {
    env_var("DEFAULT_PERMISSION")
        .and_then(|default_permission| serde_json::from_str(&default_permission).ok())
        .unwrap_or(DefaultPermission {
            container: Permission {
                process: None,
                disk: None,
                bind: None,
                device: None,
                extra_args: Some(vec![
                    "--network-veth".to_string(),
                    "--private-users=pick".to_string(),
                ]),
            },
            virtual_machine: Permission {
                process: None,
                disk: None,
                bind: None,
                device: None,
                extra_args: Some(vec!["--network-tap".to_string()]),
            },
        })
}
