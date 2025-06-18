use std::{
    fs,
    path::{Path, PathBuf},
};

use actix_identity::Identity;
use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    file::models::{
        CreateDirectory, Directory, File, ReadDirectory, ReadFile, RemoveDirectory, RemoveFile,
        WriteFile,
    },
    utils::{env::containerstate, error::ResponseError},
};

#[get("/read_file/{scope}")]
async fn read_file(
    user: Identity,
    path: web::Path<String>,
    file: web::Query<ReadFile>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

#[post("/write_file/{scope}")]
async fn write_file(
    user: Identity,
    path: web::Path<String>,
    file: web::Json<WriteFile>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

#[post("/remove_file/{scope}")]
async fn remove_file(
    user: Identity,
    path: web::Path<String>,
    file: web::Json<RemoveFile>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

#[get("/read_directory/{scope}")]
async fn read_directory(
    user: Identity,
    path: web::Path<String>,
    dir: web::Query<ReadDirectory>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

#[post("/create_directory/{scope}")]
async fn create_directory(
    user: Identity,
    path: web::Path<String>,
    dir: web::Json<CreateDirectory>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

#[post("/remove_directory/{scope}")]
async fn remove_directory(
    user: Identity,
    path: web::Path<String>,
    dir: web::Json<RemoveDirectory>,
) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

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

fn get_path(scope: &str, path_from_root: &str) -> PathBuf {
    if scope == "host" {
        Path::new(path_from_root).to_path_buf()
    } else {
        containerstate()
            .join(scope)
            .join(remove_first_slash(path_from_root))
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
