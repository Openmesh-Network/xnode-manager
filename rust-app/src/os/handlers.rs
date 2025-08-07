use std::{
    fs::{read_to_string, write},
    path::Path,
    process::Command,
};

use actix_web::{HttpResponse, Responder, get, post, web};

use crate::{
    os::models::{OSChange, OSConfiguration},
    request::{handlers::return_request_id, models::RequestIdResult},
    utils::{
        command::{CommandExecutionMode, execute_command},
        env::{nix, nixosrebuild, osdir, systemd},
        error::ResponseError,
    },
};

#[get("/get")]
async fn get() -> impl Responder {
    let flake: String;
    let flake_lock: String;

    let osdir = osdir();
    let path = Path::new(&osdir);
    {
        let path = path.join("flake.nix");
        match read_to_string(&path) {
            Ok(file) => {
                flake = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading OS flake config from {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }
    {
        let path = path.join("flake.lock");
        match read_to_string(&path) {
            Ok(file) => {
                flake_lock = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading OS flake lock from {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }

    HttpResponse::Ok().json(OSConfiguration {
        flake,
        flake_lock,
        xnode_owner: read_to_string(path.join("xnode-owner")).ok(),
        domain: read_to_string(path.join("domain")).ok(),
        acme_email: read_to_string(path.join("acme-email")).ok(),
        user_passwd: read_to_string(path.join("user-passwd")).ok(),
    })
}

#[post("/set")]
async fn set(change: web::Json<OSChange>) -> impl Responder {
    return_request_id(Box::new(move |request_id| {
        log::info!("Performing OS change: {:?}", change);
        let osdir = osdir();
        let path = Path::new(&osdir);

        if let Some(flake) = &change.flake {
            let path = path.join("flake.nix");
            if let Err(e) = write(&path, flake) {
                return RequestIdResult::Error {
                    error: format!("Error writing OS flake to {}: {}", path.display(), e),
                };
            }
        }

        for (name, content) in [
            ("flake.nix", &change.flake),
            ("xnode-owner", &change.xnode_owner),
            ("domain", &change.domain),
            ("acme-email", &change.acme_email),
            ("user-passwd", &change.user_passwd),
        ] {
            if let Some(content) = content {
                let path = path.join(name);
                if let Err(e) = write(&path, content) {
                    return RequestIdResult::Error {
                        error: format!("Error writing {} to {}: {}", content, path.display(), e),
                    };
                }
            }
        }

        if let Some(update_inputs) = &change.update_inputs {
            let mut command = Command::new(format!("{}nix", nix()));
            command
                .env("NIX_REMOTE", "daemon")
                .arg("flake")
                .arg("update");
            for input in update_inputs {
                command.arg(input);
            }
            command.arg("--flake").arg(path);
            if let Err(e) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
                return RequestIdResult::Error {
                    error: format!("Error updating OS flake: {}", e),
                };
            }
        }

        let mut command = Command::new(format!("{}nixos-rebuild", nixosrebuild()));
        command
            .env("NIX_REMOTE", "daemon")
            .arg("switch")
            .arg("--flake")
            .arg(path);
        if let Err(e) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
            return RequestIdResult::Error {
                error: format!("Error switching to new OS config: {}", e),
            };
        }

        RequestIdResult::Success { body: None }
    }))
}

#[post("/reboot")]
async fn reboot() -> impl Responder {
    return_request_id(Box::new(move |request_id| {
        let mut command = Command::new(format!("{}systemctl", systemd()));
        command.arg("reboot");
        if let Err(e) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
            return RequestIdResult::Error {
                error: format!("Error rebooting OS: {}", e),
            };
        }

        RequestIdResult::Success { body: None }
    }))
}
