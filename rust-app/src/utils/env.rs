use std::{
    env::var,
    path::{Path, PathBuf},
};

use log::{error, warn};

fn env_var(id: &str) -> Option<String> {
    var(id)
        .inspect_err(|e| {
            warn!("Could not read env var {}: {}", id, e);
        })
        .ok()
}

pub fn hostname() -> String {
    env_var("HOSTNAME").unwrap_or(String::from("0.0.0.0"))
}

pub fn port() -> String {
    env_var("PORT").unwrap_or(String::from("34391"))
}

pub fn owner() -> String {
    env_var("OWNER")
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or(String::from("eth:0000000000000000000000000000000000000000"))
}

pub fn datadir() -> PathBuf {
    env_var("DATADIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/xnode-manager").to_path_buf())
}

pub fn osdir() -> String {
    env_var("DATADIR").unwrap_or(String::from("/etc/nixos"))
}

pub fn authdir() -> PathBuf {
    env_var("AUTHDIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("auth"))
}

pub fn containerdir() -> PathBuf {
    env_var("CONTAINERDIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("containers"))
}

pub fn backupdir() -> PathBuf {
    env_var("BACKUPDIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("backups"))
}

pub fn buildcores() -> u64 {
    env_var("BUILDCORES")
        .and_then(|s| {
            str::parse::<u64>(&s)
                .inspect_err(|e| {
                    error!("Could not parse BUILDCORES to u64: {}", e);
                })
                .ok()
        })
        .unwrap_or(0)
}

pub fn nix() -> String {
    env_var("NIX").unwrap_or(String::from(""))
}

pub fn systemd() -> String {
    env_var("SYSTEMD").unwrap_or(String::from(""))
}

pub fn e2fsprogs() -> String {
    env_var("E2FSPROGS").unwrap_or(String::from(""))
}
