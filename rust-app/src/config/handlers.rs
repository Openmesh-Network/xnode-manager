use std::{
    fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write},
    path::{self, PathBuf},
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
        command::{execute_command, CommandOutputError},
        env::{
            buildcores, containerconfig, containerprofile, containersettings, containerstate,
            e2fsprogs, nix, systemd,
        },
        error::ResponseError,
        string::between,
    },
};

use super::models::{ConfigurationAction, ContainerConfiguration};

#[get("/containers")]
async fn containers(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containersettings();
    match read_dir(&path) {
        Ok(dir) => {
            let response: Vec<String> = dir
                .filter_map(|f| f.ok().and_then(|f| f.file_name().into_string().ok()))
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Could not read container dir {}: {}",
            path.display(),
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
    let path = containersettings().join(&container_id);

    let flake: String;
    let flake_lock: String;
    let mut network: Option<String> = None;

    {
        let path = path.join("flake.nix");
        match read_to_string(&path) {
            Ok(file) => {
                flake = file;
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Could not read container flake config {}: {}",
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
                    "Could not read container flake lock {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }
    {
        let path = containerconfig().join(format!("{}.conf", &container_id));
        match read_to_string(&path) {
            Ok(file) => {
                if let Some(network_zone) = between(&file, "--network-zone=", " ") {
                    network = Some(network_zone.to_string());
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Could not read container flake lock {}: {}",
                    path.display(),
                    e
                )));
            }
        }
    }

    HttpResponse::Ok().json(ContainerConfiguration {
        flake,
        flake_lock,
        network,
    })
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
                settings,
                update_inputs,
            } => {
                let path = containersettings().join(&container_id);
                if let Err(e) = create_dir_all(&path) {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error creating container folder {}: {}",
                        path.display(),
                        e
                    )));
                }
                log::info!("Created container dir {}", path.display());

                {
                    let path = path.join("flake.nix");
                    if let Err(e) = write(&path, settings.flake) {
                        return HttpResponse::InternalServerError().json(ResponseError::new(
                            format!(
                                "Error writing container flake config {}: {}",
                                path.display(),
                                e
                            ),
                        ));
                    }
                    log::info!("Created container flake {}", path.display());
                }

                if let Some(update_inputs) = update_inputs {
                    if let Some(response) = container_command(
                        &container_id,
                        ContainerCommand::FlakeUpdate {
                            flake: &path,
                            inputs: update_inputs,
                        },
                    ) {
                        return response;
                    }
                }

                if let Some(response) = container_command(
                    &container_id,
                    ContainerCommand::Create {
                        flake: &path,
                        network: settings.network,
                    },
                ) {
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

                let path = containersettings().join(&container_id);
                if let Err(e) = remove_dir_all(&path) {
                    return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error deleting container folder config {}: {}",
                        path.display(),
                        e
                    )));
                }
                log::info!("Deleted container dir {}", path.display());
            }
        }
    }

    HttpResponse::Ok().finish()
}

#[derive(Debug, PartialEq)]
enum ContainerCommand<'a> {
    Create {
        flake: &'a PathBuf,
        network: Option<String>,
    },
    Destroy,
    FlakeUpdate {
        flake: &'a PathBuf,
        inputs: Vec<String>,
    },
}
/// Performs a single CLI (std::process) command
fn container_command(container_id: &String, command: ContainerCommand) -> Option<HttpResponse> {
    log::info!("Performing {:?} on container {}", command, container_id);
    let command_name: &str;

    let cli_command = match command {
        ContainerCommand::Create { flake, ref network } => {
            command_name = "creating";
            let system: PathBuf = match build_config(flake) {
                Ok(build_folder) => build_folder,
                Err(e) => {
                    return Some(e);
                }
            };
            if let Some(e) = create_conf_file(container_id, network) {
                return Some(e);
            }
            if let Some(e) = create_state_dir(container_id) {
                return Some(e);
            }
            if let Some(e) = create_profile(container_id, system) {
                return Some(e);
            }

            let mut cli_command = Command::new(format!("{}systemctl", systemd()));
            cli_command
                .arg("reload-or-restart")
                .arg(format!("container@{}", container_id));

            cli_command
        }
        ContainerCommand::Destroy => {
            command_name = "destroying";

            let mut cli_command = Command::new(format!("{}machinectl", systemd()));
            cli_command.arg("terminate").arg(container_id);

            cli_command
        }
        ContainerCommand::FlakeUpdate { flake, ref inputs } => {
            command_name = "flake updating";

            let mut cli_command = Command::new(format!("{}nix", nix()));
            cli_command
                .env("NIX_REMOTE", "daemon")
                .arg("flake")
                .arg("update");
            for input in inputs {
                cli_command.arg(input);
            }
            cli_command.arg("--flake").arg(flake);

            cli_command
        }
    };

    if let Err(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output, e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error {} nixos container {}: Output could not be decoded: {}. Output: {:?}",
                        command_name, container_id, e, output,
                    ),
                )));
            }
            CommandOutputError::OutputError(output) => {
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

    // Post command execution actions
    if command == ContainerCommand::Destroy {
        if let Some(e) = delete_profile(container_id) {
            return Some(e);
        }
        if let Some(e) = delete_state_dir(container_id) {
            return Some(e);
        }
        if let Some(e) = delete_conf_file(container_id) {
            return Some(e);
        }
    }

    None
}

fn build_config(flake: &path::Path) -> Result<PathBuf, HttpResponse> {
    let build_folder = flake.join("build");

    let mut cli_command = Command::new(format!("{}nix", nix()));
    cli_command
        .env("NIX_REMOTE", "daemon")
        .env("NIX_BUILD_CORES", buildcores().to_string())
        .arg("build")
        .arg("-o")
        .arg(&build_folder)
        .arg(format!(
            "{}#nixosConfigurations.container.config.system.build.toplevel",
            flake.to_string_lossy()
        ));

    if let Err(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output, e) => {
                return Err(
                    HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error building configuration {}: Output could not be decoded: {}. Output: {:?}",
                        flake.display(),
                        e,
                        output,
                    ))),
                );
            }
            CommandOutputError::OutputError(output) => {
                return Err(
                    HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error building configuration {}: {}",
                        flake.display(),
                        output,
                    ))),
                );
            }
            CommandOutputError::CommandError(e) => {
                return Err(
                    HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error building configuration {}: {}",
                        flake.display(),
                        e
                    ))),
                );
            }
        };
    }

    Ok(build_folder)
}

fn create_profile(container_id: &str, system: PathBuf) -> Option<HttpResponse> {
    let container_profile = containerprofile().join(container_id);
    log::info!("Creating profile {}", container_profile.display());

    if let Err(e) = create_dir_all(&container_profile) {
        return Some(
            HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error creating nixos container profile directory {}: {}",
                container_profile.display(),
                e
            ))),
        );
    }

    let mut cli_command = Command::new(format!("{}nix-env", nix()));
    cli_command
        .env("NIX_REMOTE", "daemon")
        .arg("-p")
        .arg(container_profile.join("system"))
        .arg("--set")
        .arg(&system);

    if let Err(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output, e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error setting configuration {} for profile {}: Output could not be decoded: {}. Output: {:?}",
                        system.display(),
                        container_profile.display(),
                        e,
                        output,
                    ),
                )));
            }
            CommandOutputError::OutputError(output) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error setting configuration {} for profile {}: {}",
                        system.display(),
                        container_profile.display(),
                        output,
                    ),
                )));
            }
            CommandOutputError::CommandError(e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error setting configuration {} for profile {}: {}",
                        system.display(),
                        container_profile.display(),
                        e
                    ),
                )));
            }
        };
    };

    None
}
fn delete_profile(container_id: &str) -> Option<HttpResponse> {
    let container_profile = containerprofile().join(container_id);
    if let Err(e) = remove_dir_all(&container_profile) {
        return Some(
            HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error deleting nixos profile configuration {}: {}",
                container_profile.display(),
                e
            ))),
        );
    }

    None
}

fn create_state_dir(container_id: &str) -> Option<HttpResponse> {
    let state_dir = containerstate().join(container_id);
    log::info!("Creating state dir {}", state_dir.display());

    if let Err(e) = create_dir_all(&state_dir) {
        return Some(
            HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error creating nixos container state directory {}: {}",
                state_dir.display(),
                e
            ))),
        );
    }

    None
}
fn delete_state_dir(container_id: &str) -> Option<HttpResponse> {
    let state_dir = containerstate().join(container_id);

    // /var/empty is immutable, preventing deletion
    let mut cli_command = Command::new(format!("{}chattr", e2fsprogs()));
    cli_command
        .arg("-i")
        .arg(state_dir.join("var").join("empty"));

    if let Err(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output, e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error making {} mutable: Output could not be decoded: {}. Output: {:?}",
                        state_dir.display(),
                        e,
                        output,
                    ),
                )));
            }
            CommandOutputError::OutputError(output) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!("Error making {} mutable: {}", state_dir.display(), output,),
                )));
            }
            CommandOutputError::CommandError(e) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!("Error making {} mutable: {}", state_dir.display(), e),
                )));
            }
        };
    };

    if remove_dir_all(&state_dir).is_err() {
        // Ignore first error: Directory not empty (os error 39)
        if let Err(e) = remove_dir_all(&state_dir) {
            return Some(
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Error deleting nixos container state directory {}: {}",
                    state_dir.display(),
                    e
                ))),
            );
        }
    }

    None
}

fn create_conf_file(container_id: &str, network: &Option<String>) -> Option<HttpResponse> {
    let conf_file = containerconfig().join(format!("{}.conf", container_id));
    log::info!("Creating conf file {}", conf_file.display());

    let conf_content = if let Some(network_zone) = network {
        format!("EXTRA_NSPAWN_FLAGS=\"--network-zone={} \"", network_zone)
    } else {
        "".to_string()
    };

    if let Err(e) = write(&conf_file, conf_content) {
        return Some(
            HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error writing nixos container configuration file {}: {}",
                conf_file.display(),
                e
            ))),
        );
    }

    None
}
fn delete_conf_file(container_id: &str) -> Option<HttpResponse> {
    let conf_file = containerconfig().join(format!("{}.conf", container_id));
    if let Err(e) = remove_file(&conf_file) {
        return Some(
            HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error deleting nixos container configuration file {}: {}",
                conf_file.display(),
                e
            ))),
        );
    }

    None
}
