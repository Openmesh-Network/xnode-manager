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
        env::osdir,
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

    HttpResponse::Ok().json(OSConfiguration { flake, flake_lock })
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

    if let Some(update_inputs) = &os.update_inputs {
        let mut command = Command::new("nix");
        command.arg("flake").arg("update");
        for input in update_inputs {
            command.arg(input);
        }
        command.arg("--flake").arg(path);
        if let Some(err) = execute_command(command) {
            match err {
                CommandOutputError::OutputErrorRaw(output) => {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error updating OS flake: Output could not be decoded: {:?}",
                        output,
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

    let mut command = Command::new("nixos-rebuild");
    command.arg("switch").arg("--flake").arg(path);
    if let Some(err) = execute_command(command) {
        match err {
            CommandOutputError::OutputErrorRaw(output) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error switching to new OS config: Output could not be decoded: {:?}",
                    output,
                )));
            }
            CommandOutputError::OutputError(output) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error switching to new OS config: {}",
                    output,
                )));
            }
            CommandOutputError::CommandError(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error switching to new OS config: {}",
                    e,
                )));
            }
        };
    }

    HttpResponse::Ok().finish()
}
