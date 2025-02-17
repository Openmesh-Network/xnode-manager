use std::{
    fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, write},
    path::PathBuf,
    process::Command,
};

use actix_identity::Identity;
use actix_web::{
    get, post,
    web::{self, Path},
    HttpResponse, Responder,
};

use crate::{
    auth::{models::Scope, utils::has_permission},
    utils::{
        command::{command_output_errors, CommandOutputError},
        env::containerdir,
        error::ResponseError,
    },
};

use super::models::{ConfigurationAction, ContainerConfiguration};

#[get("/containers")]
async fn containers(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    match read_dir(containerdir()) {
        Ok(dir) => {
            let response: Vec<String> = dir
                .filter_map(|f| f.ok().and_then(|f| f.file_name().into_string().ok()))
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Could not read container dir {}: {}",
            containerdir().display(),
            e
        ))),
    }
}

#[get("/container/{container}")]
async fn container(user: Identity, path: Path<String>) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    let container_id = path.into_inner();
    let path = containerdir().join(container_id).join("flake.nix");
    match read_to_string(&path) {
        Ok(file) => HttpResponse::Ok().json(ContainerConfiguration { flake: file }),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Could not read container file {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/change")]
async fn change(user: Identity, changes: web::Json<Vec<ConfigurationAction>>) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    log::info!("Executing changes: {:?}", changes);
    for action in changes.into_inner() {
        match action {
            ConfigurationAction::Set {
                container: container_id,
                config,
            } => {
                let path = containerdir().join(&container_id);
                if let Err(e) = create_dir_all(&path) {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error creating container folder {}: {}",
                        path.display(),
                        e
                    )));
                }
                {
                    let path = path.join("flake.nix");
                    if let Err(e) = write(&path, config.flake) {
                        return HttpResponse::InternalServerError().json(ResponseError::new(
                            format!(
                                "Error writing container flake config {}: {}",
                                path.display(),
                                e
                            ),
                        ));
                    }
                }

                if let Some(response) =
                    container_command(&container_id, ContainerCommand::Create { flake: &path })
                {
                    return response;
                }
            }
            ConfigurationAction::Remove {
                container: container_id,
                backup: _backup,
            } => {
                if let Some(response) = container_command(&container_id, ContainerCommand::Destroy)
                {
                    return response;
                }

                let path = containerdir().join(&container_id);
                if let Err(e) = remove_dir_all(&path) {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error deleting container folder config {}: {}",
                        path.display(),
                        e
                    )));
                }
            }
            ConfigurationAction::Update {
                container: container_id,
                inputs,
            } => {
                let path = containerdir().join(&container_id);
                if let Some(response) = container_command(
                    &container_id,
                    ContainerCommand::FlakeUpdate {
                        flake: &path,
                        inputs,
                    },
                ) {
                    return response;
                }
            }
        }
    }

    HttpResponse::Ok().finish()
}

#[derive(Debug, PartialEq)]
enum ContainerCommand<'a> {
    Create {
        flake: &'a PathBuf,
    },
    Update {
        flake: &'a PathBuf,
    },
    Start,
    Destroy,
    FlakeUpdate {
        flake: &'a PathBuf,
        inputs: Vec<String>,
    },
}
fn container_command(container_id: &String, command: ContainerCommand) -> Option<HttpResponse> {
    log::info!("Performing {:?} on container {}", command, container_id);
    let command_name: &str;
    let command_cli = match command {
        ContainerCommand::Create { flake } => {
            command_name = "creating";
            Command::new("nixos-container")
                .arg("create")
                .arg(container_id)
                .arg("--flake")
                .arg(flake)
                .output()
        }
        ContainerCommand::Update { flake } => {
            command_name = "updating";

            Command::new("nixos-container")
                .arg("update")
                .arg(container_id)
                .arg("--flake")
                .arg(flake)
                .output()
        }
        ContainerCommand::Start => {
            command_name = "starting";
            Command::new("nixos-container")
                .arg("start")
                .arg(container_id)
                .output()
        }
        ContainerCommand::Destroy => {
            command_name = "destroying";
            Command::new("nixos-container")
                .arg("destroy")
                .arg(container_id)
                .output()
        }
        ContainerCommand::FlakeUpdate { flake, ref inputs } => {
            command_name = "flake updating";

            let mut base_cmd = Command::new("nix");
            base_cmd.arg("flake").arg("update").arg(container_id);
            for input in inputs {
                base_cmd.arg(input);
            }
            base_cmd.arg("--flake").arg(flake).output()
        }
    };

    if let Some(err) = command_output_errors(command_cli) {
        match err {
            CommandOutputError::OutputErrorRaw(output) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error {} nixos container {}: Output could not be decoded: {:?}",
                        command_name, container_id, output,
                    ),
                )));
            }
            CommandOutputError::OutputError(output) => {
                if let ContainerCommand::Create { flake } = command {
                    if output == format!("/run/current-system/sw/bin/nixos-container: container ‘{}’ already exists\n", container_id) {
                        return container_command(container_id, ContainerCommand::Update { flake });
                    }
                }

                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error {} nixos container {}: {}",
                        command_name, container_id, output,
                    ),
                )));
            }
            CommandOutputError::CommandError(e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error {} nixos container {}: {}",
                        command_name, container_id, e
                    ),
                )));
            }
        };
    }

    if let ContainerCommand::Create { flake: _ } = command {
        // nixos-container does not support creating containers without private network
        if let Some(response) = patch_container_file(container_id) {
            return Some(response);
        }

        // Start container after creation
        return container_command(container_id, ContainerCommand::Start);
    }

    None
}

fn patch_container_file(container_id: &String) -> Option<HttpResponse> {
    let path = std::path::Path::new("/etc/nixos-containers").join(format!("{}.conf", container_id));
    match read_to_string(&path) {
        Ok(container_conf) => {
            let new_conf: Vec<String> = container_conf
                .split("\n")
                .map(|l| {
                    if l.starts_with("PRIVATE_NETWORK=") {
                        return String::from("PRIVATE_NETWORK=0");
                    } else if l.starts_with("HOST_ADDRESS=") {
                        return String::from("HOST_ADDRESS=");
                    } else if l.starts_with("LOCAL_ADDRESS=") {
                        return String::from("LOCAL_ADDRESS=");
                    }

                    l.to_string()
                })
                .collect();
            if let Err(e) = write(&path, new_conf.join("\n")) {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error writing nixos container configuration file {}: {}",
                        path.display(),
                        e
                    ),
                )));
            }
        }
        Err(e) => {
            return Some(
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error reading nixos container configuration file {}: {}",
                    path.display(),
                    e
                ))),
            );
        }
    }

    None
}
