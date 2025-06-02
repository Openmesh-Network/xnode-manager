use std::fs;

use actix_identity::Identity;
use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    auth::{models::Scope, utils::has_permission},
    file::models::{
        CreateDirectory, Directory, File, ReadDirectory, ReadFile, RemoveDirectory, RemoveFile,
        WriteFile,
    },
    utils::{env::containerstate, error::ResponseError},
};

#[post("/read_file")]
async fn read_file(user: Identity, file: web::Json<ReadFile>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerstate()
        .join(&file.location.container)
        .join(remove_first_slash(&file.location.path));
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

#[post("/write_file")]
async fn write_file(user: Identity, file: web::Json<WriteFile>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerstate()
        .join(&file.location.container)
        .join(remove_first_slash(&file.location.path));
    match fs::write(&path, &file.content) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error writing file at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/remove_file")]
async fn remove_file(user: Identity, file: web::Json<RemoveFile>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerstate()
        .join(&file.location.container)
        .join(remove_first_slash(&file.location.path));
    match fs::remove_file(&path) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error removing file at path {}: {}",
            path.display(),
            e
        ))),
    }
}

#[post("/read_directory")]
async fn read_directory(user: Identity, dir: web::Json<ReadDirectory>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path: std::path::PathBuf = containerstate()
        .join(&dir.location.container)
        .join(remove_first_slash(&dir.location.path));
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

#[post("/create_directory")]
async fn create_directory(user: Identity, dir: web::Json<CreateDirectory>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerstate()
        .join(&dir.location.container)
        .join(remove_first_slash(&dir.location.path));
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

#[post("/remove_directory")]
async fn remove_directory(user: Identity, dir: web::Json<RemoveDirectory>) -> impl Responder {
    if !has_permission(user, Scope::File) {
        return HttpResponse::Unauthorized().finish();
    }

    let path = containerstate()
        .join(&dir.location.container)
        .join(remove_first_slash(&dir.location.path));
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

fn remove_first_slash(string: &str) -> &str {
    let mut chars = string.chars();

    if let Some(char) = chars.next() {
        if char != '/' {
            return string;
        }
    }

    chars.as_str()
}
