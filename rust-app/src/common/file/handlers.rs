use std::{
    io::ErrorKind,
    os::unix::fs::{MetadataExt, chown},
    path::Path,
};

use posix_acl::{ACL_EXECUTE, ACL_READ, ACL_WRITE, PosixACL, Qualifier};
use tokio::fs;

use crate::common::{
    btrfs::filesystem::du,
    response::{ResponseError, ResponseResult, TypedResponseError},
};

use super::models::{Entity, Folder, Metadata, Permission, Size};

pub async fn metadata(path: impl AsRef<Path>) -> ResponseResult<Metadata> {
    let path = path.as_ref();

    fs::metadata(path)
        .await
        .map(|metadata| {
            if metadata.is_dir() {
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
                .unwrap_or(ResponseError::new(format!(
                    "Could not get metadata of {path}: {e}",
                    path = path.display()
                )))
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

pub async fn read_file(path: impl AsRef<Path>) -> ResponseResult<Vec<u8>> {
    let path = path.as_ref();

    fs::read(path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or(ResponseError::new(format!(
                "Could not read file {path}: {e}",
                path = path.display()
            )))
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
            .unwrap_or(ResponseError::new(format!(
                "Could not write file {path}: {e}",
                path = path.display()
            )))
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
                .unwrap_or(ResponseError::new(format!(
                    "Could not remove file {path}: {e}",
                    path = path.display()
                )))
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

pub async fn read_folder(path: impl AsRef<Path>) -> ResponseResult<Folder> {
    let path = path.as_ref();

    let mut folders = vec![];
    let mut files = vec![];
    let mut symlinks = vec![];

    let mut entries = fs::read_dir(&path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or(ResponseError::new(format!(
                "Could not read folder {path}: {e}",
                path = path.display()
            )))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        ResponseError::new(format!(
            "Could not get next read folder {path}: {e}",
            path = path.display()
        ))
    })? {
        let file_type = entry.file_type().await.map_err(|e| {
            ResponseError::new(format!(
                "Could not get read folder {path} entry file type of {entry}: {e}",
                path = path.display(),
                entry = entry.file_name().display()
            ))
        })?;
        let file_name = entry
            .file_name()
            .into_string()
            .unwrap_or("NON-UNICODE NAME".to_string());

        if file_type.is_dir() {
            folders.push(file_name);
        } else if file_type.is_file() {
            files.push(file_name);
        } else if file_type.is_symlink() {
            symlinks.push(file_name);
        }
    }

    Ok(Folder {
        folders,
        files,
        symlinks,
    })
}

pub async fn create_folder(path: impl AsRef<Path>) -> ResponseResult<()> {
    let path = path.as_ref();

    fs::create_dir_all(path).await.map_err(|e| {
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or(ResponseError::new(format!(
                "Could not create folder {path}: {e}",
                path = path.display()
            )))
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
                .unwrap_or(ResponseError::new(format!(
                    "Could not remove folder {path}: {e}",
                    path = path.display()
                )))
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
    let folder = read_folder(source).await?;
    for file in folder.files {
        copy_file(source.join(&file), destination.join(&file)).await?;
    }
    for folder in folder.folders {
        copy_folder(source.join(&folder), destination.join(&folder)).await?;
    }
    Ok(())
}

pub async fn get_permissions(path: impl AsRef<Path>) -> ResponseResult<Vec<Permission>> {
    let path = path.as_ref();

    let (owner_user, owner_group) = fs::metadata(path)
        .await
        .map(|metadata| (metadata.uid(), metadata.gid()))
        .map_err(|e| {
            TypedResponseError::from_io(&e, path)
                .map(ResponseError::typed)
                .unwrap_or(ResponseError::new(format!(
                    "Could not get owner user and group of {path}: {e}",
                    path = path.display()
                )))
        })?;
    let permissions = PosixACL::read_acl(path).map_err(|e| {
        ResponseError::new(format!(
            "Could not get permissions of {path}: {e}",
            path = path.display()
        ))
    })?;

    Ok(permissions
        .entries()
        .into_iter()
        .filter(|permission| !matches!(permission.qual, Qualifier::Mask))
        .map(|permission| Permission {
            granted_to: match permission.qual {
                Qualifier::UserObj => Entity::User(owner_user),
                Qualifier::GroupObj => Entity::Group(owner_group),
                Qualifier::Other => Entity::Any,
                Qualifier::User(id) => Entity::User(id),
                Qualifier::Group(id) => Entity::Group(id),
                _ => Entity::Unknown,
            },
            read: permission.perm & ACL_READ != 0,
            write: permission.perm & ACL_WRITE != 0,
            execute: permission.perm & ACL_EXECUTE != 0,
        })
        .collect::<Vec<Permission>>())
}

pub async fn set_permissions(
    path: impl AsRef<Path>,
    permissions: impl AsRef<[Permission]>,
) -> ResponseResult<()> {
    let path = path.as_ref();
    let permissions = permissions.as_ref();

    let owner_user = permissions
        .iter()
        .find_map(|permission| match permission.granted_to {
            Entity::User(id) => Some(id),
            _ => None,
        })
        .ok_or_else(|| ResponseError::new(format!(
            "Could not set permission on {path}: No user permission (one is required).",
            path = path.display()
        )))?;
    let owner_group = permissions
        .iter()
        .find_map(|permission| match permission.granted_to {
            Entity::Group(id) => Some(id),
            _ => None,
        })
        .ok_or_else( || ResponseError::new(format!(
            "Could not set permission on {path}: No group permission (one is required).",
            path = path.display()
        )))?;

    chown(path, Some(owner_user), Some(owner_group)).map_err(|e| 
        TypedResponseError::from_io(&e, path)
            .map(ResponseError::typed)
            .unwrap_or(ResponseError::new(format!(
                "Could not set permission on {path}: Ownership transfer to {owner_user}:{owner_group} failed:  {e}",
                path = path.display()
            )))
    )?;

    let mut acl = PosixACL::empty();
    for permission in permissions {
        let mut perm = 0;
        if permission.read {
            perm |= ACL_READ;
        }
        if permission.write {
            perm |= ACL_WRITE;
        }
        if permission.execute {
            perm |= ACL_EXECUTE;
        }
        match permission.granted_to {
            Entity::User(id) => {
                if id == owner_user {
                    acl.set(Qualifier::UserObj, perm);
                } else {
                    acl.set(Qualifier::User(id), perm);
                }
            }
            Entity::Group(id) => {
                if id == owner_group {
                    acl.set(Qualifier::GroupObj, perm);
                } else {
                    acl.set(Qualifier::Group(id), perm);
                }
            }
            Entity::Any => {
                acl.set(Qualifier::Other, perm);
            }
            Entity::Unknown => {}
        };
    }

    acl.write_acl(path).map_err(|e| {
        ResponseError::new(format!(
            "Could not set permission on {path}: Writing permissions to file failed: {e}",
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
