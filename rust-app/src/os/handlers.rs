use std::{
    fs::{exists, read_to_string, write},
    path::Path,
    process::Command,
};

use actix_identity::Identity;
use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    os::models::OSConfiguration,
    utils::{
        command::{command_output_errors, CommandOutputError},
        env::osdir,
        error::ResponseError,
    },
};

#[post("/get")]
async fn get(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::OS) {
        return HttpResponse::Unauthorized().finish();
    }

    let flake: String;
    let owner: String;

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
        let path = path.join("xnode-owner");
        match exists(&path) {
            Ok(file_exists) => {
                if file_exists {
                    match read_to_string(&path) {
                        Ok(file) => {
                            owner = file;
                        }
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(ResponseError::new(
                                format!("Error reading xnode owner from {}: {}", path.display(), e),
                            ));
                        }
                    }
                } else {
                    // Nix module default value
                    owner = String::from("eth:0000000000000000000000000000000000000000");
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error checking existence of xnode owner file at {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }

    HttpResponse::Ok().json(OSConfiguration { flake, owner })
}

#[post("/set")]
async fn set(user: Identity, os: web::Json<OSConfiguration>) -> impl Responder {
    if !has_permission(user, Scope::OS) {
        return HttpResponse::Unauthorized().finish();
    }

    log::info!("Performing OS update: {:?}", os);
    let osdir = osdir();
    let path = Path::new(&osdir);
    {
        let path = path.join("flake.nix");
        if let Err(e) = write(&path, &os.flake) {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error writing OS flake to {}: {}",
                path.display(),
                e
            )));
        }
    }
    {
        let path = path.join("xnode-owner");
        if let Err(e) = write(&path, &os.owner) {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error writing xnode owner to {}: {}",
                path.display(),
                e
            )));
        }
    }

    let command = Command::new("nixos-rebuild")
        .arg("switch")
        .arg("--recreate-lock-file")
        .arg("--flake")
        .arg(format!("{}#xnode", osdir))
        .output();
    if let Some(err) = command_output_errors(command) {
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
