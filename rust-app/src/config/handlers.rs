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
        env::{buildcores, containerdir, e2fsprogs, nix, systemd},
        error::ResponseError,
    },
};

use super::models::{ConfigurationAction, ContainerConfiguration};

#[get("/containers")]
async fn containers(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerdir();
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
    let path = containerdir().join(container_id);

    let flake: String;
    let flake_lock: String;

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

    HttpResponse::Ok().json(ContainerConfiguration { flake, flake_lock })
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
            } => {
                let path = containerdir().join(&container_id);
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
                log::info!("Deleted container dir {}", path.display());
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
    Destroy,
    FlakeUpdate {
        flake: &'a PathBuf,
        inputs: Vec<String>,
    },
}
fn container_command(container_id: &String, command: ContainerCommand) -> Option<HttpResponse> {
    log::info!("Performing {:?} on container {}", command, container_id);
    let command_name: &str;

    let cli_command = match command {
        ContainerCommand::Create { flake } => {
            command_name = "creating";
            let system: PathBuf = match build_config(flake) {
                Ok(build_folder) => build_folder,
                Err(e) => {
                    return Some(e);
                }
            };
            if let Some(e) = create_conf_file(container_id) {
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

    if let Some(err) = execute_command(cli_command) {
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

    match command {
        ContainerCommand::Destroy => {
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
        ContainerCommand::FlakeUpdate { flake, inputs: _ } => {
            // Update container after flake update
            return container_command(container_id, ContainerCommand::Create { flake });
        }
        _ => {}
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

    if let Some(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output) => {
                return Err(
                    HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Error building configuration {}: Output could not be decoded: {:?}",
                        flake.display(),
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

fn profile_root() -> PathBuf {
    path::Path::new("/nix/var/nix/profiles/per-container").to_path_buf()
}
fn create_profile(container_id: &str, system: PathBuf) -> Option<HttpResponse> {
    let container_profile = profile_root().join(container_id);
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

    if let Some(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error setting configuration {} for profile {}: Output could not be decoded: {:?}",
                        system.display(),
                        container_profile.display(),
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
    let container_profile = profile_root().join(container_id);
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

fn state_root() -> PathBuf {
    path::Path::new("/var/lib/nixos-containers").to_path_buf()
}
fn create_state_dir(container_id: &str) -> Option<HttpResponse> {
    let state_dir = state_root().join(container_id);
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
    let state_dir = state_root().join(container_id);

    // /var/empty is immutable, preventing deletion
    let mut cli_command = Command::new(format!("{}chattr", e2fsprogs()));
    cli_command
        .arg("-i")
        .arg(state_dir.join("var").join("empty"));

    if let Some(err) = execute_command(cli_command) {
        match err {
            CommandOutputError::OutputErrorRaw(output) => {
                return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                    format!(
                        "Error making {} mutable: Output could not be decoded: {:?}",
                        state_dir.display(),
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

fn conf_root() -> PathBuf {
    path::Path::new("/etc/nixos-containers").to_path_buf()
}
fn create_conf_file(container_id: &str) -> Option<HttpResponse> {
    let conf_file = conf_root().join(format!("{}.conf", container_id));
    log::info!("Creating conf file {}", conf_file.display());

    if let Err(e) = write(
        &conf_file,
        "
PRIVATE_NETWORK=0
HOST_ADDRESS=
LOCAL_ADDRESS=
HOST_BRIDGE=
HOST_PORT=
AUTO_START=0
",
    ) {
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
    let conf_file = conf_root().join(format!("{}.conf", container_id));
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
