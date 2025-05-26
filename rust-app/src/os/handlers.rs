use std::{
    fs::{read_to_string, write},
    path::Path,
    process::Command,
};

use actix_identity::Identity;
use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    os::models::{OSChange, OSConfiguration},
    request::{
        handlers::return_request_id,
        models::{RequestIdResult, RequestsAppData},
    },
    utils::{
        command::{execute_command, CommandExecutionMode},
        env::{nix, nixosrebuild, osdir},
        error::ResponseError,
    },
};

#[get("/get")]
async fn get(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::OS) {
        return HttpResponse::Unauthorized().finish();
    }

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
async fn set(
    user: Identity,
    os: web::Json<OSChange>,
    data: web::Data<RequestsAppData>,
) -> impl Responder {
    if !has_permission(user, Scope::OS) {
        return HttpResponse::Unauthorized().finish();
    }

    return_request_id(
        data,
        Box::new(move |request_id| {
            log::info!("Performing OS update: {:?}", os);
            let osdir = osdir();
            let path = Path::new(&osdir);

            if let Some(flake) = &os.flake {
                let path = path.join("flake.nix");
                if let Err(e) = write(&path, flake) {
                    return RequestIdResult::Error {
                        error: format!("Error writing OS flake to {}: {}", path.display(), e),
                    };
                }
            }

            for (name, content) in [
                ("flake.nix", &os.flake),
                ("xnode-owner", &os.xnode_owner),
                ("domain", &os.domain),
                ("acme-email", &os.acme_email),
                ("user-passwd", &os.user_passwd),
            ] {
                if let Some(content) = content {
                    let path = path.join(name);
                    if let Err(e) = write(&path, content) {
                        return RequestIdResult::Error {
                            error: format!(
                                "Error writing {} to {}: {}",
                                content,
                                path.display(),
                                e
                            ),
                        };
                    }
                }
            }

            if let Some(update_inputs) = &os.update_inputs {
                let mut command = Command::new(format!("{}nix", nix()));
                command
                    .env("NIX_REMOTE", "daemon")
                    .arg("flake")
                    .arg("update");
                for input in update_inputs {
                    command.arg(input);
                }
                command.arg("--flake").arg(path);
                if let Err(err) =
                    execute_command(command, CommandExecutionMode::Stream { request_id })
                {
                    return RequestIdResult::Error {
                        error: format!("Error updating OS flake: {}", err),
                    };
                }
            }

            let mut command = Command::new(format!("{}nixos-rebuild", nixosrebuild()));
            command
                .env("NIX_REMOTE", "daemon")
                .arg("switch")
                .arg("--flake")
                .arg(path);
            match os.as_child {
                true => {
                    if let Err(e) = command.spawn() {
                        return RequestIdResult::Error {
                            error: format!("Error spawning OS switch command child: {}", e),
                        };
                    }
                }
                false => {
                    if let Err(err) =
                        execute_command(command, CommandExecutionMode::Stream { request_id })
                    {
                        return RequestIdResult::Error {
                            error: format!("Error switching to new OS config: {}", err),
                        };
                    }
                }
            }

            RequestIdResult::Success { body: None }
        }),
    )
}
