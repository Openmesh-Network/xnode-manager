use std::{
    fs,
    path::{Path, PathBuf},
};

use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{
    file::models::{
        CreateDirectory, Directory, File, ReadDirectory, ReadFile, RemoveDirectory, RemoveFile,
        WriteFile,
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
