use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{Responder, get, post, web};
use reqwest::{Body, Client};
use tokio::process::Command;

use super::models::{Backup, SendData};
use crate::{
    common::{
        btrfs::{TemporarySubvolume, receive, send, subvolume},
        command::execute_command_simple,
        env::find,
        file::{ReadFolderOptions, create_folder, metadata, r#move, read_folder, shift},
        path::get_scoped_path,
        process::{SystemCtlCommand, execute},
        response::{
            ResponseError, ResponseResult, TypedResponseError, json_response, raw_response,
        },
    },
    container::handlers::ensure_initialized,
};

#[get("")]
async fn backup_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup"]);
    read_folder(path, &ReadFolderOptions::default())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| Backup { id: item.name })
                .collect::<Vec<Backup>>()
        })
        .map(json_response)
}

#[post("/create")]
async fn create_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let root = get_scoped_path(&scope, &["data"]);
    let snapshot = get_scoped_path(&scope, &["backup", &backup]);

    let parent = get_scoped_path(&scope, &["backup"]);
    create_folder(&parent).await?;
    shift(&parent, "foreign").await?;

    subvolume::snapshot(&root, &snapshot, true)
        .await
        .map(raw_response)
}

#[post("/restore")]
async fn restore_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let root = get_scoped_path(&scope, &["data"]);
    let snapshot = get_scoped_path(&scope, &["backup", &backup]);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    r#move(
        &root,
        get_scoped_path(&scope, &["backup", &format!("pre-restore-{timestamp}")]),
    )
    .await?;

    subvolume::snapshot(&snapshot, &root, false).await?;

    // Remove empty folders of the snapshot that represent subvolume mounts
    // The empty folders prevent creation of new subvolumes on the same location
    let mut command = Command::new(format!("{}find", find()));
    command
        .arg(&root)
        .args(["-mindepth", "1", "-inum", "2", "-empty", "-delete"]);
    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not remove empty subvolume mount folders in {root}: {e}",
                root = root.display()
            ))
        })?;

    execute(
        None::<String>,
        format!("container@{container}.service"),
        SystemCtlCommand::Restart,
    )
    .await?;

    Ok(raw_response(()))
}

#[post("/remove")]
async fn remove_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup", &backup]);

    subvolume::delete(&path).await.map(raw_response)
}

#[post("/receive")]
async fn receive_endpoint(
    path: web::Path<(String, String)>,
    body: web::Payload,
) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup", &backup]);

    let parent = get_scoped_path(&scope, &["backup"]);
    create_folder(&parent).await?;
    shift(&parent, "foreign").await?;

    // Create temporary folder to receive the files in
    // Required as btrfs receive doesn't allow us to only receive a single file at a specified path
    let receive_subvolume = get_scoped_path(&scope, &[".temporary"]);
    if let Err(e) = metadata(&receive_subvolume).await
        && let Some(typed) = &e.typed_error
        && matches!(typed, TypedResponseError::PathNotFound { path: _path })
    {
        subvolume::create(&receive_subvolume).await?;
    }

    let receive_subvolume = receive_subvolume.join("backup-receive");
    if let Err(e) = metadata(&receive_subvolume).await
        && let Some(typed) = &e.typed_error
        && matches!(typed, TypedResponseError::PathNotFound { path: _path })
    {
        subvolume::create(&receive_subvolume).await?;
    }

    let receive_subvolume = TemporarySubvolume::create(receive_subvolume.join(&backup)).await?;

    receive(receive_subvolume.path(), body).await?;

    // Take the first file from the folder and move it into our desired path location
    let file = read_folder(receive_subvolume.path(), &ReadFolderOptions::default())
        .await?
        .into_iter()
        .next()
        .map(|item| item.name)
        .ok_or_else(|| ResponseError::new("Nothing received."))?;

    subvolume::snapshot(receive_subvolume.path().join(&file), &path, true).await?;

    Ok(json_response(()))
}

#[post("/send")]
async fn send_endpoint(
    path: web::Path<(String, String)>,
    data: web::Json<SendData>,
) -> ResponseResult<impl Responder> {
    let (container, backup) = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container".to_string(), container.clone()];
    let path = get_scoped_path(&scope, &["backup", &backup]);

    // Create temporary folder to send the files
    // This allows for proper referencing to common backups on the receiver side
    // Also prevents subvolume deletion while the transfer is active
    let send_subvolume = get_scoped_path(&scope, &[".temporary"]);
    if let Err(e) = metadata(&send_subvolume).await
        && let Some(typed) = &e.typed_error
        && matches!(typed, TypedResponseError::PathNotFound { path: _path })
    {
        subvolume::create(&send_subvolume).await?;
    }

    let send_subvolume = send_subvolume.join("backup-send");
    if let Err(e) = metadata(&send_subvolume).await
        && let Some(typed) = &e.typed_error
        && matches!(typed, TypedResponseError::PathNotFound { path: _path })
    {
        subvolume::create(&send_subvolume).await?;
    }

    subvolume::snapshot(&path, &send_subvolume, true).await?;
    let send_subvolume = TemporarySubvolume::new(send_subvolume.join(&backup));

    let (stream, mut child) = send(
        send_subvolume.path().to_path_buf(),
        data.common
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(move |backup| get_scoped_path(&scope, &["backup", &backup])),
    )
    .await?;

    let response = Client::default()
        .post(&data.receive)
        .body(Body::wrap_stream(stream))
        .send()
        .await
        .map_err(|e| {
            ResponseError::new(format!(
                "Failed to stream to receive endpoint {receive}: {e}",
                receive = data.receive
            ))
        })?;

    let endpoint_status = response.status();
    if !endpoint_status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ResponseError::new(format!(
            "Receive endpoint {receive} returned {endpoint_status}: {body}",
            receive = data.receive
        )));
    }

    let process_status = child
        .wait()
        .await
        .map_err(|e| ResponseError::new(format!("btrfs send process error: {e}")))?;
    if !process_status.success() {
        let mut logs = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut logs).await;
        }
        return Err(ResponseError::new(format!(
            "btrfs send process failed: {code:?} {logs}",
            code = process_status.code()
        )));
    };

    Ok(json_response(()))
}
