use actix_web::{Responder, post, web};
use reqwest::{Body, Client};

use super::models::SendData;
use crate::{
    common::{
        btrfs::{receive, send},
        file::{create_folder, shift},
        path::get_scoped_path,
        response::{ResponseError, ResponseResult, raw_response},
    },
    container::handlers::ensure_initialized,
};

#[post("/receive")]
async fn receive_endpoint(
    path: web::Path<String>,
    body: web::Payload,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup"]);

    create_folder(&path).await?;
    shift(&path, "foreign").await?;

    receive(&path, body).await.map(raw_response)
}

#[post("/send")]
async fn send_endpoint(
    path: web::Path<String>,
    data: web::Json<SendData>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let subvolumes: Vec<_> = data
        .send
        .iter()
        .map(|subvolume| get_scoped_path(&scope, &["backup", subvolume]))
        .collect();
    let common: Vec<_> = data
        .common
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|backup| get_scoped_path(&scope, &["backup", backup]))
        .collect();

    let (stream, mut child) = send(subvolumes, common).await?;

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

    Ok(raw_response(()))
}
