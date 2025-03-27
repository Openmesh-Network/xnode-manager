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
    utils::{
        command::{execute_command, CommandOutputError},
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
        xnode_owner: read_to_string(path.join("xnode_owner")).ok(),
        domain: read_to_string(path.join("domain")).ok(),
        acme_email: read_to_string(path.join("acme_email")).ok(),
        user_passwd: read_to_string(path.join("user_passwd")).ok(),
    })
}

#[post("/set")]
async fn set(user: Identity, os: web::Json<OSChange>) -> impl Responder {
    if !has_permission(user, Scope::OS) {
        return HttpResponse::Unauthorized().finish();
    }

    log::info!("Performing OS update: {:?}", os);
    let osdir = osdir();
    let path = Path::new(&osdir);

    if let Some(flake) = &os.flake {
        let path = path.join("flake.nix");
        if let Err(e) = write(&path, flake) {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error writing OS flake to {}: {}",
                path.display(),
                e
            )));
        }
    }

    for (name, content) in [
        ("flake.nix", &os.flake),
        ("xnode_owner", &os.xnode_owner),
        ("domain", &os.domain),
        ("acme_email", &os.acme_email),
        ("user_passwd", &os.user_passwd),
    ] {
        if let Some(content) = content {
            let path = path.join(name);
            if let Err(e) = write(&path, content) {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error writing {} to {}: {}",
                    content,
                    path.display(),
                    e
                )));
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
        if let Err(err) = execute_command(command) {
            match err {
                CommandOutputError::OutputErrorRaw(output, e) => {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error updating OS flake: Output could not be decoded: {}. Output: {:?}",
                        e, output,
                    )));
                }
                CommandOutputError::OutputError(output) => {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error updating OS flake: {}",
                        output,
                    )));
                }
                CommandOutputError::CommandError(e) => {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error updating OS flake: {}",
                        e,
                    )));
                }
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
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error spawning OS switch command child: {}",
                    e,
                )));
            }
        }
        false => {
            if let Err(err) = execute_command(command) {
                match err {
                    CommandOutputError::OutputErrorRaw(output, e) => {
                        return HttpResponse::InternalServerError().json(ResponseError::new(
                            format!(
                            "Error switching to new OS config: Output could not be decoded: {}. Output: {:?}",
                            e,
                            output,
                        ),
                        ));
                    }
                    CommandOutputError::OutputError(output) => {
                        return HttpResponse::InternalServerError().json(ResponseError::new(
                            format!("Error switching to new OS config: {}", output,),
                        ));
                    }
                    CommandOutputError::CommandError(e) => {
                        return HttpResponse::InternalServerError().json(ResponseError::new(
                            format!("Error switching to new OS config: {}", e,),
                        ));
                    }
                };
            }
        }
    }

    HttpResponse::Ok().finish()
}
