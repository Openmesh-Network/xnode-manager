use std::{
    io::ErrorKind,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use futures::future::join_all;
use tokio::{fs, process::Command};

use crate::common::{
    btrfs::filesystem::du,
    command::execute_command_simple,
    env::systemd,
    response::{ResponseError, ResponseResult, TypedResponseError},
    string::escaped_utf8_from_bytes,
};

use super::{
    ReadFolderOptions,
    models::{FolderItem, Metadata, Size},
};

pub async fn metadata(path: impl AsRef<Path>) -> ResponseResult<Metadata> {
    let path = path.as_ref();

    fs::symlink_metadata(path)
        .await
        .map(|metadata| {
            if metadata.is_symlink() {
                Metadata::Link {}
            } else if metadata.is_dir() {
                Metadata::Folder {}
            } else if metadata.is_file() {
                Metadata::File {}
            } else {
                Metadata::Unknown {}
            }
        })
        .map_err(|e| {
            TypedResponseError::from_io(&e, path)
                .map(ResponseError::typed)
                .unwrap_or_else(|| {
                    ResponseError::new(format!(
                        "Could not get metadata of {path}: {e}",
                        path = path.display()
                    ))
                })
        })
}

pub async fn size(path: impl AsRef<Path>) -> ResponseResult<Size> {
    let path = path.as_ref();

    du(path).await.map(|du| Size {
        exclusive: du.exclusive,
        shared: du.shared,
    })
}

pub async fn r#move(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    if let Some(parent) = destination.parent() {
        create_folder(parent).await?;
    }
    fs::rename(source, destination).await.map_err(|e| {
        ResponseError::new(format!(
            "Could not move {source} to {destination}: {e}",
            source = source.display(),
            destination = destination.display()
        ))
    })
}

pub async fn remove(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();
    match metadata(path).await? {
        Metadata::File {} | Metadata::Link {} => remove_file(path).await,
        Metadata::Folder {} => remove_folder(path).await,
        Metadata::Unknown {} => Err(ResponseError::new(format!(
            "Could not determine type of {path}",
            path = path.display()
        ))),
    }
}

pub async fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    match metadata(source).await? {
        Metadata::File {} | Metadata::Link {} => copy_file(source, destination).await,
        Metadata::Folder {} => copy_folder(source, destination).await,
        Metadata::Unknown {} => Err(ResponseError::new(format!(
            "Could not determine type of {source}",
            source = source.display()
        ))),
    }
}

pub async fn read_file(path: impl AsRef<Path>) -> ResponseResult<Vec<u8>> {
    let path = path.as_ref();

    fs::read(path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or_else(|| {
                ResponseError::new(format!(
                    "Could not read file {path}: {e}",
                    path = path.display()
                ))
            })
    })
}

pub async fn write_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> ResponseResult<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        create_folder(parent).await?;
    }
    fs::write(path, content).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or_else(|| {
                ResponseError::new(format!(
                    "Could not write file {path}: {e}",
                    path = path.display()
                ))
            })
    })
}

pub async fn remove_file(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();
    fs::remove_file(path)
        .await
        .or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                // Treat file not found as remove success
                return Ok(());
            }

            Err(e)
        })
        .map_err(|e| {
            TypedResponseError::from_io(&e, path)
                .map(ResponseError::typed)
                .unwrap_or_else(|| {
                    ResponseError::new(format!(
                        "Could not remove file {path}: {e}",
                        path = path.display()
                    ))
                })
        })
}

pub async fn copy_file(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    if let Some(parent) = destination.parent() {
        create_folder(parent).await?;
    }
    fs::copy(source, destination)
        .await
        .map(|_copied_bytes| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not copy file {source} to {destination}: {e}",
                source = source.display(),
                destination = destination.display()
            ))
        })
}

pub async fn read_folder(
    path: impl AsRef<Path>,
    options: &ReadFolderOptions,
) -> ResponseResult<Vec<FolderItem>> {
    let path = path.as_ref();

    let mut entries = fs::read_dir(&path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or_else(|| {
                ResponseError::new(format!(
                    "Could not read folder {path}: {e}",
                    path = path.display()
                ))
            })
    })?;

    let mut items = vec![];

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        ResponseError::new(format!(
            "Could not get next read folder {path}: {e}",
            path = path.display()
        ))
    })? {
        let item_name = escaped_utf8_from_bytes(entry.file_name().as_bytes());
        let path = path.join(&item_name);

        let get = async move {
            let mut item_metadata = None;
            if options.metadata.unwrap_or(false) {
                item_metadata = metadata(&path).await.ok();
            }

            let mut item_size = None;
            if options.size.unwrap_or(false) {
                item_size = size(&path).await.ok();
            }

            FolderItem {
                name: item_name,
                metadata: item_metadata,
                size: item_size,
            }
        };

        items.push(get);
    }

    Ok(join_all(items).await)
}

pub async fn create_folder(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();

    fs::create_dir_all(path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or_else(|| {
                ResponseError::new(format!(
                    "Could not create folder {path}: {e}",
                    path = path.display()
                ))
            })
    })
}

pub async fn remove_folder(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();

    fs::remove_dir_all(path)
        .await
        .or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                // Treat folder not found as remove success
                return Ok(());
            }

            Err(e)
        })
        .map_err(|e| {
            TypedResponseError::from_io(&e, path)
                .map(ResponseError::typed)
                .unwrap_or_else(|| {
                    ResponseError::new(format!(
                        "Could not remove folder {path}: {e}",
                        path = path.display()
                    ))
                })
        })
}

#[async_recursion::async_recursion]
pub async fn copy_folder<SOURCE, DESTINATION>(
    source: SOURCE,
    destination: DESTINATION,
) -> ResponseResult<()>
where
    SOURCE: AsRef<Path> + Send + Sync,
    DESTINATION: AsRef<Path> + Send + Sync,
{
    let source = source.as_ref();
    let destination = destination.as_ref();

    create_folder(destination).await?;
    let folder = read_folder(
        source,
        &ReadFolderOptions {
            metadata: Some(true),
            ..Default::default()
        },
    )
    .await?;
    for item in folder {
        match item.metadata {
            Some(Metadata::File {}) | Some(Metadata::Link {}) => {
                copy_file(source.join(&item.name), destination.join(&item.name)).await?;
            }
            Some(Metadata::Folder {}) => {
                copy_folder(source.join(&item.name), destination.join(&item.name)).await?;
            }
            Some(Metadata::Unknown {}) | None => {
                return Err(ResponseError::new(format!(
                    "Could not get metadata of {name}",
                    name = item.name
                )));
            }
        };
    }

    Ok(())
}

pub async fn read_link(path: impl AsRef<Path>) -> ResponseResult<PathBuf> {
    let path = path.as_ref();

    fs::read_link(path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or_else(|| {
                ResponseError::new(format!(
                    "Could not read link {path}: {e}",
                    path = path.display()
                ))
            })
    })
}

pub async fn write_link(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> ResponseResult<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    fs::symlink(source, destination).await.map_err(|e| {
        ResponseError::new(format!(
            "Could not write link {destination} to {source}: {e}",
            source = source.display(),
            destination = destination.display()
        ))
    })
}

pub async fn shift(path: impl AsRef<Path>, range: impl AsRef<str>) -> ResponseResult<()> {
    let path = path.as_ref();
    let range = range.as_ref();

    let mut command = Command::new(format!("{}systemd-dissect", systemd()));
    command.arg("--shift").arg(path).arg(range);

    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| ())
        .map_err(|e| {
            ResponseError::new(format!(
                "Could not shift {path} to {range}: {e}",
                path = path.display()
            ))
        })
}

pub fn remove_first_slash(string: &str) -> &str {
    let mut chars = string.chars();

    if let Some(char) = chars.next()
        && char != '/'
    {
        return string;
    }

    chars.as_str()
}
