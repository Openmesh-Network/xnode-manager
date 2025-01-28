use std::{
    env::var,
    path::{Path, PathBuf},
};

pub fn hostname() -> String {
    var("HOSTNAME").ok().unwrap_or(String::from("0.0.0.0"))
}

pub fn port() -> String {
    var("PORT").ok().unwrap_or(String::from("34391"))
}

pub fn owner() -> String {
    var("OWNER")
        .ok()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or(String::from("eth:0000000000000000000000000000000000000000"))
}

pub fn datadir() -> PathBuf {
    var("DATADIR")
        .ok()
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/xnode-manager").to_path_buf())
}

pub fn osdir() -> String {
    var("DATADIR").ok().unwrap_or(String::from("/etc/nixos"))
}

pub fn containerdir() -> PathBuf {
    var("CONTAINERDIR")
        .ok()
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("containers"))
}

pub fn backupdir() -> PathBuf {
    var("BACKUPDIR")
        .ok()
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("backups"))
}
