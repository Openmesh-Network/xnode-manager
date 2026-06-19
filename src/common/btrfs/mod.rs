use std::{fmt::Display, path::Path, process::Stdio};

use futures::{Stream, StreamExt};
use tokio::process::{Child, Command};
use tokio_util::io::ReaderStream;

use crate::common::{env::btrfs, response::ResponseError};

use super::response::ResponseResult;

pub mod filesystem;
pub mod qgroup;
pub mod quota;
pub mod subvolume;

pub async fn receive(
    directory: impl AsRef<Path>,
    mut send: impl Stream<Item = Result<impl AsRef<[u8]>, impl Display>> + Unpin,
) -> ResponseResult<()> {
    let directory = directory.as_ref();

    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["receive", "--chroot"]);
    command.arg(directory);

    command.stdin(Stdio::piped());
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| ResponseError::new(format!("Failed to spawn btrfs receive: {e}")))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| ResponseError::new("btrfs receive doesn't have an stdin attached"))?;

    while let Some(chunk) = send.next().await {
        let chunk = chunk
            .map_err(|e| ResponseError::new(format!("Failed to read btrfs receive stream: {e}")))?;
        tokio::io::AsyncWriteExt::write_all(&mut stdin, chunk.as_ref())
            .await
            .map_err(|e| ResponseError::new(format!("Failed to write to btrfs receive: {e}")))?;
    }
    drop(stdin);

    let status = child
        .wait()
        .await
        .map_err(|e| ResponseError::new(format!("btrfs receive process error: {e}")))?;

    if !status.success() {
        let mut logs = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut logs).await;
        }
        return Err(ResponseError::new(format!(
            "btrfs receive process failed: {code:?} {logs}",
            code = status.code()
        )));
    };

    Ok(())
}

pub async fn send<S, C>(
    subvolumes: S,
    common: C,
) -> ResponseResult<(
    impl Stream<Item = std::io::Result<actix_web::web::Bytes>>,
    Child,
)>
where
    S: IntoIterator,
    S::Item: AsRef<Path>,
    C: IntoIterator,
    C::Item: AsRef<Path>,
{
    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["send", "-e"]);
    for subvolume in common {
        command.arg("-c").arg(subvolume.as_ref());
    }
    for subvolume in subvolumes {
        command.arg(subvolume.as_ref());
    }

    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| ResponseError::new(format!("Failed to spawn btrfs send: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ResponseError::new("Failed to capture btrfs send stdout"))?;

    Ok((ReaderStream::new(stdout), child))
}
