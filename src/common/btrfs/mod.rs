use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Stdio,
};

use futures::{Stream, StreamExt};
use tokio::process::{Child, Command};
use tokio_util::io::ReaderStream;

use crate::common::{env::btrfs, response::ResponseError};

use super::{file::create_folder, response::ResponseResult};

pub mod filesystem;
pub mod qgroup;
pub mod quota;
pub mod subvolume;

pub struct ReceiveFolder {
    path: PathBuf,
}

impl ReceiveFolder {
    pub async fn create(path: impl AsRef<Path>) -> ResponseResult<Self> {
        create_folder(&path).await?;
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReceiveFolder {
    fn drop(&mut self) {
        let path = self.path.clone();
        tokio::spawn(async move {
            if let Ok(mut entries) = tokio::fs::read_dir(&path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let _ = subvolume::delete(entry.path()).await;
                }
            }
            let _ = tokio::fs::remove_dir(&path).await;
        });
    }
}

pub async fn receive(
    path: impl AsRef<Path>,
    mut send: impl Stream<Item = Result<impl AsRef<[u8]>, impl Display>> + Unpin,
) -> ResponseResult<()> {
    let path = path.as_ref();

    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.args(["receive", "--chroot"]);
    command.arg(path);

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

pub async fn send(
    path: impl AsRef<Path>,
    common: impl Iterator<Item = impl AsRef<Path>>,
) -> ResponseResult<(
    impl Stream<Item = std::io::Result<actix_web::web::Bytes>>,
    Child,
)> {
    let path = path.as_ref();

    let mut command = Command::new(format!("{}btrfs", btrfs()));
    command.arg("send");
    for parent in common {
        command.arg("-c").arg(parent.as_ref());
    }
    command.arg(path);

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
