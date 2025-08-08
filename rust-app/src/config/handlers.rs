use std::{
    fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write},
    path::PathBuf,
    process::Command,
};

use actix_web::{HttpResponse, Responder, get, post, web};

use crate::{
    config::models::ContainerChange,
    request::{
        handlers::return_request_id,
        models::{RequestId, RequestIdResult},
    },
    utils::{
        command::{CommandExecutionMode, execute_command},
        env::{
            buildcores, containerconfig, containerprofile, containersettings, containerstate,
            e2fsprogs, nix, systemd, systemdconfig,
        },
        error::ResponseError,
        fs::copy_dir_all,
        string::between,
    },
};

use super::models::ContainerConfiguration;

#[get("/containers")]
async fn containers() -> impl Responder {
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

#[get("/container/{container}/get")]
async fn get(path: web::Path<String>) -> impl Responder {
    let container_id = path.into_inner();
    let path = containersettings().join(&container_id);

    let flake: String;
    let mut flake_lock: Option<String> = None;
    let mut network: Option<String> = None;
    let mut nvidia_gpus: Option<Vec<u64>> = None;

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
                log::warn!(
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
                if let Some(nspawn_flags) = between(&file, "\"", "\"") {
                    if nspawn_flags.contains("nvidia") {
                        nvidia_gpus = Some(vec![]);
                    }

                    nspawn_flags.split(" ").for_each(|flag| {
                        if flag.starts_with("--network-zone=") {
                            network = Some(flag.replace("--network-zone=", ""));
                        }

                        if flag.starts_with("--bind-ro=") {
                            let path = flag.replace("--bind-ro=", "");
                            if path.starts_with("/dev/nvidia") {
                                let device = path.replace("/dev/nvidia", "");
                                if !["ctl", "-modeset", "-uvm", "-uvm-tools"]
                                    .contains(&device.as_str())
                                {
                                    match device.parse::<u64>() {
                                        Ok(device_id) => {
                                            nvidia_gpus.get_or_insert(vec![]).push(device_id);
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Could not parse nvidia device id {} to u64: {}",
                                                device,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    });
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
        nvidia_gpus,
    })
}

#[post("/container/{container}/set")]
async fn set(path: web::Path<String>, change: web::Json<ContainerChange>) -> impl Responder {
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

            if let Err(e) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
                return RequestIdResult::Error {
                    error: format!(
                        "Error flake updating nixos container {}: {}",
                        container_id, e
                    ),
                };
            }
        }

        if let Some(e) = create_conf_file(
            &container_id,
            &change.settings.network,
            &change.settings.nvidia_gpus,
            request_id,
        ) {
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

        if let Err(e) = execute_command(command, CommandExecutionMode::Stream { request_id }) {
            return RequestIdResult::Error {
                error: format!("Error creating nixos container {}: {}", container_id, e),
            };
        }

        RequestIdResult::Success { body: None }
    }))
}

#[post("/container/{container}/remove")]
async fn remove(path: web::Path<String>) -> impl Responder {
    return_request_id(Box::new(move |request_id| {
        let container_id = path.into_inner();
        let mut command = Command::new(format!("{}systemctl", systemd()));
        command
            .arg("stop")
            .arg(format!("container@{}", container_id));

        // Should be inside if "container running", then fail on error can be added back
        let _ = execute_command(command, CommandExecutionMode::Stream { request_id });

        if let Some(e) = remove_profile(&container_id) {
            let ignore = if let RequestIdResult::Error { error } = &e {
                error.ends_with("No such file or directory (os error 2)")
            } else {
                false
            };

            if !ignore {
                return e;
            }
        }
        if let Some(e) = remove_state_dir(&container_id, request_id) {
            return e;
        }
        if let Some(e) = remove_conf_file(&container_id, request_id) {
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
        log::info!("removed container dir {}", path.display());

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

    if let Err(e) = execute_command(cli_command, CommandExecutionMode::Stream { request_id }) {
        return Some(RequestIdResult::Error {
            error: format!("Error building configuration {}: {}", flake.display(), e),
        });
    }

    None
}
fn remove_profile(container_id: &str) -> Option<RequestIdResult> {
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
fn remove_state_dir(container_id: &str, request_id: RequestId) -> Option<RequestIdResult> {
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

fn create_conf_file(
    container_id: &str,
    network: &Option<String>,
    nvidia_gpus: &Option<Vec<u64>>,
    request_id: RequestId,
) -> Option<RequestIdResult> {
    let conf_file = containerconfig().join(format!("{}.conf", container_id));
    log::info!("Creating conf file {}", conf_file.display());

    let nspawn_flags: Vec<String> = []
        .into_iter()
        .chain(
            network
                .as_ref()
                .map(|network_zone| vec![format!("--network-zone={}", network_zone)])
                .unwrap_or_default(),
        )
        .chain(
            nvidia_gpus
                .as_ref()
                .map(|gpus| {
                    gpus.iter()
                        .map(|gpu_id| gpu_id.to_string())
                        .chain(
                            ["ctl", "-modeset", "-uvm", "-uvm-tools"]
                                .into_iter()
                                .map(|str| str.to_string()),
                        )
                        .map(|postfix| format!("--bind-ro=/dev/nvidia{}", postfix))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default(),
        )
        .collect();

    if let Err(e) = write(
        &conf_file,
        format!("EXTRA_NSPAWN_FLAGS=\"{}\"", nspawn_flags.join(" ")),
    ) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error writing nixos container configuration file {}: {}",
                conf_file.display(),
                e
            ),
        });
    }

    let systemd_conf_file = systemdconfig()
        .join(format!("container@{}.service.d", container_id))
        .join("99-XnodeManager.conf");
    log::info!("Creating systemd conf file {}", conf_file.display());

    if let Some(dir) = systemd_conf_file.parent() {
        if let Err(e) = create_dir_all(dir) {
            return Some(RequestIdResult::Error {
                error: format!(
                    "Error creating nixos container systemd configuration folder {}: {}",
                    dir.display(),
                    e
                ),
            });
        }
    }

    let systemd_config: Vec<String> = ["[Service]"]
        .into_iter()
        .map(|str| str.to_string())
        .chain(
            nvidia_gpus
                .as_ref()
                .map(|gpus| {
                    gpus.iter()
                        .map(|gpu_id| gpu_id.to_string())
                        .chain(
                            ["ctl", "-caps*", "-modeset", "-uvm", "-uvm-tools"]
                                .into_iter()
                                .map(|str| str.to_string()),
                        )
                        .map(|postfix| format!("DeviceAllow=/dev/nvidia{} rw", postfix))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default(),
        )
        .collect();

    if let Err(e) = write(&systemd_conf_file, systemd_config.join("\n")) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error writing nixos container systemd configuration file {}: {}",
                systemd_conf_file.display(),
                e
            ),
        });
    }

    let mut reload_command = Command::new(format!("{}systemctl", systemd()));
    reload_command.arg("daemon-reload");
    if let Err(e) = execute_command(reload_command, CommandExecutionMode::Stream { request_id }) {
        return Some(RequestIdResult::Error {
            error: format!("Error reloading systemd daemon: {}", e),
        });
    }

    None
}

fn remove_conf_file(container_id: &str, request_id: RequestId) -> Option<RequestIdResult> {
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

    let systemd_conf_file = systemdconfig()
        .join(format!("container@{}.service.d", container_id))
        .join("99-XnodeManager.conf");
    if let Err(e) = remove_file(&systemd_conf_file) {
        return Some(RequestIdResult::Error {
            error: format!(
                "Error deleting nixos container systemd configuration file {}: {}",
                systemd_conf_file.display(),
                e
            ),
        });
    }

    let mut reload_command = Command::new(format!("{}systemctl", systemd()));
    reload_command.arg("daemon-reload");
    if let Err(e) = execute_command(reload_command, CommandExecutionMode::Stream { request_id }) {
        return Some(RequestIdResult::Error {
            error: format!("Error reloading systemd daemon: {}", e),
        });
    }

    None
}
