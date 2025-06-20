use std::{
    fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write},
    path::PathBuf,
    process::Command,
};

use actix_identity::Identity;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::warn;

use crate::{
    auth::{models::Scope, utils::has_permission},
    config::models::ContainerChange,
    request::{
        handlers::return_request_id,
        models::{RequestId, RequestIdResult},
    },
    utils::{
        command::{execute_command, CommandExecutionMode},
        env::{
            buildcores, containerconfig, containerprofile, containersettings, containerstate,
            e2fsprogs, nix, systemd,
        },
        error::ResponseError,
        fs::copy_dir_all,
        string::between,
    },
};

use super::models::ContainerConfiguration;

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
async fn container(user: Identity, path: web::Path<String>) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    let container_id = path.into_inner();
    let path = containersettings().join(&container_id);

    let flake: String;
    let mut flake_lock: Option<String> = None;
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
                flake_lock = Some(file);
            }
            Err(e) => {
                warn!(
                    "Could not read container flake lock {}: {}",
                    path.display(),
                    e
                );
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
                    "Could not read container config {}: {}",
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

#[post("/container/{container}/change")]
async fn change(
    user: Identity,
    path: web::Path<String>,
    change: web::Json<ContainerChange>,
) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    return_request_id(Box::new(move |request_id| {
        let container_id = path.into_inner();
        let path = containersettings().join(&container_id);
        if let Err(e) = create_dir_all(&path) {
            return RequestIdResult::Error {
                error: format!("Error creating container folder {}: {}", path.display(), e),
            };
        }
        log::info!("Created container dir {}", path.display());

        {
            let path = path.join("flake.nix");
            if let Err(e) = write(&path, &change.settings.flake) {
                return RequestIdResult::Error {
                    error: format!(
                        "Error writing container flake config {}: {}",
                        path.display(),
                        e
                    ),
                };
            }
            log::info!("Created container flake {}", path.display());
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
            command.arg("--flake").arg(&path);

            if let Err(err) = execute_command(command, CommandExecutionMode::Stream { request_id })
            {
                return RequestIdResult::Error {
                    error: format!(
                        "Error flake updating nixos container {}: {}",
                        container_id, err
                    ),
                };
            }
        }

        if let Some(e) = create_conf_file(&container_id, &change.settings.network) {
            return e;
        }
        if let Some(e) = create_state_dir(&container_id) {
            return e;
        }
        if let Some(e) = create_profile(path, &container_id, request_id) {
            return e;
        }

        let mut command = Command::new(format!("{}systemctl", systemd()));
        command
            .arg("reload-or-restart")
            .arg(format!("container@{}", container_id));

        if let Err(err) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
            return RequestIdResult::Error {
                error: format!("Error creating nixos container {}: {}", container_id, err),
            };
        }

        RequestIdResult::Success { body: None }
    }))
}

#[post("/container/{container}/delete")]
async fn delete(user: Identity, path: web::Path<String>) -> impl Responder {
    if !has_permission(user, Scope::Config) {
        return HttpResponse::Unauthorized().finish();
    }

    return_request_id(Box::new(move |request_id| {
        let container_id = path.into_inner();
        let mut command = Command::new(format!("{}systemctl", systemd()));
        command
            .arg("stop")
            .arg(format!("container@{}", container_id));

        // Should be inside if "container running", then fail on error can be added back
        let _ = execute_command(command, CommandExecutionMode::Stream { request_id });

        if let Some(e) = delete_profile(&container_id) {
            let ignore = if let RequestIdResult::Error { error } = &e {
                error.ends_with("No such file or directory (os error 2)")
            } else {
                false
            };

            if !ignore {
                return e;
            }
        }
        if let Some(e) = delete_state_dir(&container_id, request_id) {
            return e;
        }
        if let Some(e) = delete_conf_file(&container_id) {
            return e;
        }

        let path = containersettings().join(&container_id);
        if let Err(e) = remove_dir_all(&path) {
            return RequestIdResult::Error {
                error: format!(
                    "Error deleting container folder config {}: {}",
                    path.display(),
                    e
                ),
            };
        }
        log::info!("Deleted container dir {}", path.display());

        RequestIdResult::Success { body: None }
    }))
}

fn create_profile(
    flake: PathBuf,
    container_id: &str,
    request_id: RequestId,
) -> Option<RequestIdResult> {
    let container_profile = containerprofile().join(container_id);
    log::info!("Creating profile {}", container_profile.display());

    if let Err(e) = create_dir_all(&container_profile) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error creating nixos profile {}: {}",
                container_profile.display(),
                e
            ),
        });
    }

    let mut cli_command = Command::new(format!("{}nix", nix()));
    cli_command
        .env("NIX_REMOTE", "daemon")
        .env("NIX_BUILD_CORES", buildcores().to_string())
        .arg("build")
        .arg("--profile")
        .arg(container_profile.join("system"))
        .arg(format!(
            "{}#nixosConfigurations.container.config.system.build.toplevel",
            flake.to_string_lossy()
        ));

    if let Err(err) = execute_command(cli_command, CommandExecutionMode::Stream { request_id }) {
        return Some(RequestIdResult::Error {
            error: format!("Error building configuration {}: {}", flake.display(), err,),
        });
    }

    None
}
fn delete_profile(container_id: &str) -> Option<RequestIdResult> {
    let container_profile = containerprofile().join(container_id);
    if let Err(e) = remove_dir_all(&container_profile) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error deleting nixos profile {}: {}",
                container_profile.display(),
                e
            ),
        });
    }

    None
}

fn create_state_dir(container_id: &str) -> Option<RequestIdResult> {
    let state_dir = containerstate().join(container_id);
    log::info!("Creating state dir {}", state_dir.display());

    if let Err(e) = create_dir_all(&state_dir) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error creating nixos container state directory {}: {}",
                state_dir.display(),
                e
            ),
        });
    }

    // Create xnode-config in container state
    let xnode_config_in_dir = state_dir.join("xnode-config");
    if let Err(e) = create_dir_all(&xnode_config_in_dir) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error creating container state xnode-config directory {}: {}",
                xnode_config_in_dir.display(),
                e
            ),
        });
    }

    // Create xnode-config in container config
    let xnode_config_out_dir = containersettings().join(container_id).join("xnode-config");
    if let Err(e) = create_dir_all(&xnode_config_out_dir) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error creating container config xnode-config directory {}: {}",
                xnode_config_out_dir.display(),
                e
            ),
        });
    }

    // Set container host platform to same as host
    let mut get_host_platform = Command::new("uname");
    get_host_platform.arg("-m");
    match execute_command(get_host_platform, CommandExecutionMode::Simple) {
        Ok(mut bytes) => {
            let path = xnode_config_out_dir.join("host-platform");
            let mut postfix: Vec<u8> = "-linux".into();
            bytes.pop(); // remove newline
            bytes.append(&mut postfix);
            if let Err(e) = write(&path, bytes) {
                return Some(RequestIdResult::Error {
                    error: format!("Error writing host platform to {}: {}", path.display(), e),
                });
            }
        }
        Err(e) => {
            return Some(RequestIdResult::Error {
                error: format!("Error getting host platform: {}", e),
            });
        }
    }

    // Set container hostname to it's container id
    {
        let path = xnode_config_out_dir.join("hostname");
        if let Err(e) = write(&path, container_id) {
            return Some(RequestIdResult::Error {
                error: format!("Error writing hostname to {}: {}", path.display(), e),
            });
        }
    }

    // Copy all other files from the xnode-config dir in the container, including state version
    if let Err(e) = copy_dir_all(&xnode_config_in_dir, &xnode_config_out_dir) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error copying {} to {}: {}",
                xnode_config_in_dir.display(),
                xnode_config_out_dir.display(),
                e
            ),
        });
    }

    None
}
fn delete_state_dir(container_id: &str, request_id: RequestId) -> Option<RequestIdResult> {
    let state_dir = containerstate().join(container_id);

    // /var/empty is immutable, preventing deletion
    let mut cli_command = Command::new(format!("{}chattr", e2fsprogs()));
    cli_command
        .arg("-i")
        .arg(state_dir.join("var").join("empty"));

    let _ = execute_command(cli_command, CommandExecutionMode::Stream { request_id });

    if remove_dir_all(&state_dir).is_err() {
        // Ignore first error: Directory not empty (os error 39)
        if let Err(e) = remove_dir_all(&state_dir) {
            return Some(RequestIdResult::Error {
                error: format!(
                    "Error deleting nixos container state directory {}: {}",
                    state_dir.display(),
                    e
                ),
            });
        }
    }

    None
}

fn create_conf_file(container_id: &str, network: &Option<String>) -> Option<RequestIdResult> {
    let conf_file = containerconfig().join(format!("{}.conf", container_id));
    log::info!("Creating conf file {}", conf_file.display());

    let conf_content = if let Some(network_zone) = network {
        format!("EXTRA_NSPAWN_FLAGS=\"--network-zone={} \"", network_zone)
    } else {
        "".to_string()
    };

    if let Err(e) = write(&conf_file, conf_content) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error writing nixos container configuration file {}: {}",
                conf_file.display(),
                e
            ),
        });
    }

    None
}
fn delete_conf_file(container_id: &str) -> Option<RequestIdResult> {
    let conf_file = containerconfig().join(format!("{}.conf", container_id));
    if let Err(e) = remove_file(&conf_file) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error deleting nixos container configuration file {}: {}",
                conf_file.display(),
                e
            ),
        });
    }

    None
}
