use std::path::Path;

use tokio::process::Command;

use crate::common::{
    btrfs::qgroup::limit,
    command::execute_command_simple,
    env::systemd,
    file::{metadata, read_file, remove_file, write_file},
    path::{get_scope_root, get_scoped_path},
    process::{SystemCtlCommand, execute},
    response::{ResponseError, ResponseResult, TypedResponseError},
    string::escaped_utf8_from_bytes,
};

use super::models::{Permission, ProcessPermission};

pub async fn get_permission(
    kind: impl AsRef<str>,
    name: impl AsRef<str>,
) -> ResponseResult<Permission> {
    let kind = kind.as_ref();
    let name = name.as_ref();

    let path = get_scoped_path(&["host", "permission", kind], &["config", name]);
    let bytes = read_file(&path).await?;
    serde_json::from_str(&escaped_utf8_from_bytes(bytes)).map_err(|e| {
        ResponseError::new(format!(
            "Could not convert {path} into permission: {e}",
            path = path.display()
        ))
    })
}

pub async fn set_permission(
    permission: Permission,
    kind: impl AsRef<str>,
    name: impl AsRef<str>,
    detect_changes: bool,
    allow_restart: bool,
) -> ResponseResult<()> {
    let kind = kind.as_ref();
    let name = name.as_ref();

    let root = get_scope_root(&[kind, name]);
    let mut skip = false;

    if let Err(e) = metadata(&root).await {
        if let Some(typed) = &e.typed_error
            && matches!(typed, TypedResponseError::PathNotFound { path: _path })
        {
            // Container doesn't exist yes, limit will be applied on initialization
            skip = true;
        } else {
            return Err(e);
        }
    };

    if !skip {
        let mut disk_changed = false;
        let mut slice_changed = false;
        let mut cli_changed = false;

        if detect_changes {
            let current_permission = get_permission(kind, name).await?;

            if current_permission.disk != permission.disk {
                disk_changed = true;
            }
            if current_permission.process != permission.process {
                slice_changed = true;
            }
            if current_permission.bind != permission.bind
                || current_permission.device != permission.device
                || current_permission.extra_args != permission.extra_args
            {
                cli_changed = true;
            }
        } else {
            disk_changed = true;
            slice_changed = true;
            cli_changed = true;
        };

        if disk_changed {
            match &permission.disk {
                Some(disk) => {
                    limit(&root, disk.total).await?;
                }
                None => {
                    limit(&root, None).await?;
                }
            }
        }

        if slice_changed {
            if let Some(process) = &permission.process {
                let root = get_scoped_path(&["host", "permission", kind], &["systemd"]);
                let suffix = format!(
                    "{name}-{kind}-machine.slice",
                    name = name.replace("-", "_"),
                    kind = kind.replace("-", "_")
                );
                write_slice(root.join(&suffix), &process.total).await?;
                write_slice(root.join(format!("run-{suffix}")), &process.run).await?;
                match &process.command {
                    Some(command) => {
                        write_slice(root.join(format!("command-{suffix}")), &command.total).await?;
                        write_slice(root.join(format!("build-command-{suffix}")), &command.build)
                            .await?;
                        write_slice(
                            root.join(format!("update-command-{suffix}")),
                            &command.update,
                        )
                        .await?;
                    }
                    None => {
                        write_slice(root.join(format!("command-{suffix}")), &None).await?;
                        write_slice(root.join(format!("build-command-{suffix}")), &None).await?;
                        write_slice(root.join(format!("update-command-{suffix}")), &None).await?;
                    }
                }
            }

            let mut command = Command::new(format!("{}systemctl", systemd()));
            command.arg("daemon-reload");
            execute_command_simple(command).await.map_err(|e| {
                ResponseError::new(format!("Could not reload systemctl daemon: {e}"))
            })?;
        }

        if cli_changed {
            let path = get_scoped_path(&["host", "permission", kind], &["cli", name]);
            let args: Vec<String> =
                [].into_iter()
                    .chain(permission.bind.as_ref().into_iter().flatten().map(
                        |(destination, bind)| {
                            let property = if bind.readonly.unwrap_or(true) {
                                "--bind-ro"
                            } else {
                                "--bind"
                            };
                            match &bind.path {
                                Some(source) => format!("{property}={source}:{destination}"),
                                None => format!("{property}={destination}"),
                            }
                        },
                    ))
                    .chain(
                        permission
                            .device
                            .as_ref()
                            .and_then(|device| device.policy.as_ref())
                            .map(|policy| format!("--property=DevicePolicy={policy}")),
                    )
                    .chain(
                        permission
                            .device
                            .as_ref()
                            .and_then(|device| device.allow.as_ref())
                            .into_iter()
                            .flatten()
                            .map(|(device, allow)| {
                                let allowed = [
                                    if allow.read.unwrap_or(false) { "r" } else { "" },
                                    if allow.write.unwrap_or(false) {
                                        "w"
                                    } else {
                                        ""
                                    },
                                    if allow.mknod.unwrap_or(false) {
                                        "m"
                                    } else {
                                        ""
                                    },
                                ]
                                .join("");
                                format!("--property=DeviceAllow={device} {allowed}")
                            }),
                    )
                    .chain(
                        permission
                            .extra_args
                            .as_ref()
                            .into_iter()
                            .flatten()
                            .map(|extra_arg| extra_arg.to_string()),
                    )
                    .collect();
            write_file(path, args.join(" ")).await?;
            if allow_restart {
                execute(
                    None::<String>,
                    &format!("{kind}@{name}.service"),
                    SystemCtlCommand::Restart,
                )
                .await?;
            }
        }
    }

    let path = get_scoped_path(&["host", "permission", kind], &["config", name]);
    let bytes = serde_json::to_string(&permission).map_err(|e| {
        ResponseError::new(format!("Could not convert {permission:?} into json: {e}"))
    })?;
    write_file(path, bytes).await
}

async fn write_slice(
    path: impl AsRef<Path>,
    content: &Option<ProcessPermission>,
) -> ResponseResult<()> {
    let path = path.as_ref();
    match content {
        Some(process) => {
            let mut slice = String::new();

            if let Some(cpu) = &process.cpu {
                if let Some(weight) = &cpu.weight {
                    slice.push_str(&format!("CPUWeight={weight}\n"));
                }
                if let Some(max) = &cpu.max {
                    slice.push_str(&format!("CPUQuota={max}\n"));
                }
                if let Some(allowed_cores) = &cpu.allowed_cores {
                    slice.push_str(&format!(
                        "AllowedCPUs={allowed_cores}\n",
                        allowed_cores = allowed_cores
                            .iter()
                            .map(|core| core.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    ));
                }
            }

            if let Some(memory) = &process.memory {
                let mut accounting = false;
                if let Some(max) = &memory.max {
                    accounting = true;
                    slice.push_str(&format!("MemoryMax={max}\n"));
                }
                if let Some(soft_max) = &memory.soft_max {
                    accounting = true;
                    slice.push_str(&format!("MemoryHigh={soft_max}\n"));
                }
                if accounting {
                    slice.push_str("MemoryAccounting=true\n");
                }
            }

            if let Some(subprocess) = &process.subprocess {
                let mut accounting = false;
                if let Some(max) = &subprocess.max {
                    accounting = true;
                    slice.push_str(&format!("TasksMax={max}\n"));
                }
                if accounting {
                    slice.push_str("TasksAccounting=true\n");
                }
            }

            if let Some(io) = &process.io {
                let mut accounting = false;
                for (device, io) in io {
                    if let Some(weight) = &io.weight {
                        accounting = true;
                        slice.push_str(&format!("IODeviceWeight={device} {weight}\n"));
                    }
                    if let Some(max_bandwidth) = &io.max_bandwidth {
                        if let Some(read) = &max_bandwidth.read {
                            accounting = true;
                            slice.push_str(&format!("IOReadBandwidthMax={device} {read}\n"));
                        }
                        if let Some(write) = &max_bandwidth.write {
                            accounting = true;
                            slice.push_str(&format!("IOWriteBandwidthMax={device} {write}\n"));
                        }
                    };
                    if let Some(max_iops) = &io.max_iops {
                        if let Some(read) = &max_iops.read {
                            accounting = true;
                            slice.push_str(&format!("IOReadIOPSMax={device} {read}\n"));
                        }
                        if let Some(write) = &max_iops.write {
                            accounting = true;
                            slice.push_str(&format!("IOWriteIOPSMax={device} {write}\n"));
                        }
                    };
                }
                if accounting {
                    slice.push_str("IOAccounting=true\n");
                }
            }

            write_file(path, slice).await
        }
        None => remove_file(path).await,
    }
}
