use std::{
    fs::{self, metadata},
    os::unix::fs::{MetadataExt, chown},
    path::{Path, PathBuf},
};

use actix_web::{HttpResponse, Responder, get, post, web};
use posix_acl::{ACL_EXECUTE, ACL_READ, ACL_WRITE, PosixACL, Qualifier};

use crate::{
    file::models::{
        CreateDirectory, Directory, Entity, File, GetPermissions, Permission, ReadDirectory,
        ReadFile, RemoveDirectory, RemoveFile, SetPermissions, WriteFile,
    },
    utils::{env::containerstate, error::ResponseError},
};

#[get("/{scope}/read_file")]
async fn read_file(path: web::Path<String>, file: web::Query<ReadFile>) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &file.path);
    match fs::read(&path) {
        Ok(output) => HttpResponse::Ok().json(File {
            content: output.into(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error reading file at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/{scope}/write_file")]
async fn write_file(path: web::Path<String>, file: web::Json<WriteFile>) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &file.path);
    match fs::write(&path, &file.content) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error writing file at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/{scope}/remove_file")]
async fn remove_file(path: web::Path<String>, file: web::Json<RemoveFile>) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &file.path);
    match fs::remove_file(&path) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error removing file at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[get("/{scope}/read_directory")]
async fn read_directory(path: web::Path<String>, dir: web::Query<ReadDirectory>) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &dir.path);
    match fs::read_dir(&path) {
        Ok(content) => {
            let content_with_type = content
                .flat_map(|c| c.ok())
                .map(|c| (c.file_type(), c.file_name()));

            let mut directories = vec![];
            let mut files = vec![];
            let mut symlinks = vec![];
            let mut unknown = vec![];

            for (file_type, file_name) in content_with_type {
                match file_type {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            directories.push(file_name);
                        } else if file_type.is_file() {
                            files.push(file_name);
                        } else if file_type.is_symlink() {
                            symlinks.push(file_name);
                        }
                    }
                    Err(_) => {
                        unknown.push(file_name);
                    }
                }
            }

            let unknown_name = "UNKOWN_NAME".to_string();
            HttpResponse::Ok().json(Directory {
                directories: directories
                    .into_iter()
                    .map(|d| d.into_string().unwrap_or(unknown_name.clone()))
                    .collect(),
                files: files
                    .into_iter()
                    .map(|f| f.into_string().unwrap_or(unknown_name.clone()))
                    .collect(),
                symlinks: symlinks
                    .into_iter()
                    .map(|l| l.into_string().unwrap_or(unknown_name.clone()))
                    .collect(),
                unknown: unknown
                    .into_iter()
                    .map(|u| u.into_string().unwrap_or(unknown_name.clone()))
                    .collect(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error reading directory at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/{scope}/create_directory")]
async fn create_directory(
    path: web::Path<String>,
    dir: web::Json<CreateDirectory>,
) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &dir.path);
    let create = if dir.make_parent {
        fs::create_dir_all(&path)
    } else {
        fs::create_dir(&path)
    };
    match create {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error creating directory at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/{scope}/remove_directory")]
async fn remove_directory(
    path: web::Path<String>,
    dir: web::Json<RemoveDirectory>,
) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &dir.path);
    let create = if dir.make_empty {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_dir(&path)
    };
    match create {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error removing directory at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[get("/{scope}/get_permissions")]
async fn get_permissions(
    path: web::Path<String>,
    target: web::Query<GetPermissions>,
) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &target.path);
    let (owner_user, owner_group) = match metadata(&path) {
        Ok(metadata) => (metadata.uid(), metadata.gid()),
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting owner of path {}: {}",
                path.display(),
                e
            )));
        }
    };
    match PosixACL::read_acl(&path) {
        Ok(permissions) => HttpResponse::Ok().json(
            permissions
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
                .collect::<Vec<Permission>>(),
        ),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error getting permissions on path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/{scope}/set_permissions")]
async fn set_permissions(
    path: web::Path<String>,
    target: web::Json<SetPermissions>,
) -> impl Responder {
    let scope = path.into_inner();
    let path = get_path(&scope, &target.path);

    let owner_user =
        match target
            .permissions
            .iter()
            .find_map(|permission| match permission.granted_to {
                Entity::User(id) => Some(id),
                _ => None,
            }) {
            Some(id) => id,
            None => {
                return HttpResponse::InternalServerError().json(ResponseError::new(
                    "No user permission (one is required).".to_string(),
                ));
            }
        };
    let owner_group =
        match target
            .permissions
            .iter()
            .find_map(|permission| match permission.granted_to {
                Entity::Group(id) => Some(id),
                _ => None,
            }) {
            Some(id) => id,
            None => {
                return HttpResponse::InternalServerError().json(ResponseError::new(
                    "No group permission (one is required).".to_string(),
                ));
            }
        };

    if let Err(e) = chown(&path, Some(owner_user), Some(owner_group)) {
        return HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error setting owner {}:{} on path {}: {}",
            owner_user,
            owner_group,
            path.display(),
            e
        )));
    }

    let mut acl = PosixACL::empty();
    for permission in &target.permissions {
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
    match acl.write_acl(&path) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error setting permissions on path {}: {}",
            path.display(),
            e
        ))),
    }
}

fn get_path(scope: &str, path_from_root: &str) -> PathBuf {
    if scope.starts_with("container:") {
        containerstate()
            .join(scope.replace("container:", ""))
            .join(remove_first_slash(path_from_root))
    } else {
        Path::new(path_from_root).to_path_buf()
    }
}

fn remove_first_slash(string: &str) -> &str {
    let mut chars = string.chars();

    if let Some(char) = chars.next() {
        if char != '/' {
            return string;
        }
    }

    chars.as_str()
}
