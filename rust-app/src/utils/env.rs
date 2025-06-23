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
    env_var("HOSTNAME").unwrap_or("0.0.0.0".to_string())
}

pub fn port() -> String {
    env_var("PORT").unwrap_or("34391".to_string())
}

pub fn datadir() -> PathBuf {
    env_var("DATADIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/xnode-manager").to_path_buf())
}

pub fn osdir() -> String {
    env_var("OSDIR").unwrap_or("/etc/nixos".to_string())
}

pub fn containersettings() -> PathBuf {
    env_var("CONTAINERSETTINGS")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("containers"))
}

pub fn containerstate() -> PathBuf {
    env_var("CONTAINERSTATE")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/nixos-containers").to_path_buf())
}

pub fn containerprofile() -> PathBuf {
    env_var("CONTAINERPROFILE")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/nix/var/nix/profiles/per-container").to_path_buf())
}

pub fn containerconfig() -> PathBuf {
    env_var("CONTAINERCONFIG")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/etc/nixos-containers").to_path_buf())
}

pub fn backupdir() -> PathBuf {
    env_var("BACKUPDIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("backups"))
}

pub fn commandstream() -> PathBuf {
    env_var("COMMANDSTREAM")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("commandstream"))
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
    env_var("NIX").unwrap_or("".to_string())
}

pub fn nixosrebuild() -> String {
    env_var("NIXOSREBUILD").unwrap_or("".to_string())
}

pub fn systemd() -> String {
    env_var("SYSTEMD").unwrap_or("".to_string())
}

pub fn e2fsprogs() -> String {
    env_var("E2FSPROGS").unwrap_or("".to_string())
}
