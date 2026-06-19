use std::pin::Pin;

use actix_web::{
    Responder, post,
    web::{self, Bytes},
};
use async_compression::{
    Level,
    tokio::bufread::{ZstdDecoder, ZstdEncoder},
};
use futures::{Stream, TryStreamExt};
use reqwest::{Body, Client};
use tokio_util::io::{ReaderStream, StreamReader};

use super::models::{ReceiveQuery, SendData, SendQuery};
use crate::{
    common::{
        btrfs::{receive, send},
        file::{create_folder, shift},
        path::get_scoped_path,
        response::{ResponseError, ResponseResult, raw_response},
    },
    container::{handlers::ensure_initialized, transfer::backup::models::Compression},
};

#[post("/receive")]
async fn receive_endpoint(
    path: web::Path<String>,
    query: web::Query<ReceiveQuery>,
    body: web::Payload,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["backup"]);

    create_folder(&path).await?;
    shift(&path, "foreign").await?;

    let stream = body.map_err(std::io::Error::other);
    let stream: Pin<Box<dyn Stream<Item = std::io::Result<Bytes>>>> = match query.compression {
        Some(Compression::Zstd) => {
            let decoder = ZstdDecoder::new(StreamReader::new(stream));
            let compressed_stream = ReaderStream::new(decoder);
            Box::pin(compressed_stream)
        }
        None => Box::pin(stream),
    };
    let stream = Box::pin(stream);

    receive(&path, stream).await.map(raw_response)
}

#[post("/send")]
async fn send_endpoint(
    path: web::Path<String>,
    query: web::Query<SendQuery>,
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

    let body = match query.compression {
        Some(Compression::Zstd) => {
            let encoder = match query.compression_level {
                Some(level) => {
                    ZstdEncoder::with_quality(StreamReader::new(stream), Level::Precise(level))
                }
                None => ZstdEncoder::new(StreamReader::new(stream)),
            };
            let compressed_stream = ReaderStream::new(encoder);
            Body::wrap_stream(compressed_stream)
        }
        None => Body::wrap_stream(stream),
    };

    let response = Client::default()
        .post(&data.receive)
        .body(body)
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
