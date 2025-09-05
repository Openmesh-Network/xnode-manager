use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use actix_web::{HttpResponse, Responder, get, web};

use crate::{
    info::models::{Flake, FlakeMetadata, FlakeQuery, Group, User},
    utils::{
        command::{CommandExecutionMode, execute_command},
        env::{containerstate, nix},
        error::ResponseError,
        output::Output,
    },
};

#[get("/flake")]
async fn flake(query: web::Query<FlakeQuery>) -> impl Responder {
    let mut command = Command::new(format!("{}nix", nix()));
    command
        .env("NIX_REMOTE", "daemon")
        .arg("flake")
        .arg("metadata")
        .arg(&query.flake)
        .arg("--json")
        .arg("--no-use-registries")
        .arg("--refresh")
        .arg("--no-write-lock-file");

    match execute_command(command, CommandExecutionMode::Simple) {
        Ok(output) => match output.into() {
            Output::UTF8 { output: output_str } => {
                match serde_json::from_str::<FlakeMetadata>(&output_str) {
                    Ok(output_parsed) => HttpResponse::Ok().json(Flake {
                        last_modified: output_parsed.lastModified,
                        revision: output_parsed.revision,
                    }),
                    Err(e) => {
                        HttpResponse::InternalServerError().json(ResponseError::new(format!(
                        "Flake metadata could not be parsed to expected format: {}. Metadata: {}",
                        e, output_str
                    )))
                    }
                }
            }
            Output::Bytes { output } => {
                HttpResponse::InternalServerError().json(ResponseError::new(format!(
                    "Flake metadata could not be decoded as UTF8: {:?}.",
                    output
                )))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Error getting flake metadata of {}: {}",
            &query.flake, e
        ))),
    }
}

#[get("/users/{scope}/users")]
async fn users(path: web::Path<String>) -> impl Responder {
    let scope = path.into_inner();
    let prefix = if scope.starts_with("container:") {
        Some(containerstate().join(scope.replace("container:", "")))
    } else {
        None
    };

    match get_users(prefix) {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

#[get("/users/{scope}/groups")]
async fn groups(path: web::Path<String>) -> impl Responder {
    let scope = path.into_inner();
    let prefix = if scope.starts_with("container:") {
        Some(containerstate().join(scope.replace("container:", "")))
    } else {
        None
    };

    match get_groups(prefix) {
        Ok(groups) => HttpResponse::Ok().json(groups),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

impl FromStr for User {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(":").collect();

        let name = match split.first() {
            Some(name) => name,
            None => return Err(ResponseError::new(format!("Missing user name in {}", s))),
        };
        let id = match split.get(2) {
            Some(id) => match u32::from_str(id) {
                Ok(id) => id,
                Err(e) => {
                    return Err(ResponseError::new(format!(
                        "Could not convert user id {} to u32: {}",
                        id, e
                    )));
                }
            },
            None => return Err(ResponseError::new(format!("Missing user id in {}", s))),
        };
        let group = match split.get(3) {
            Some(group) => match u32::from_str(group) {
                Ok(group) => group,
                Err(e) => {
                    return Err(ResponseError::new(format!(
                        "Could not convert user group {} to u32: {}",
                        group, e
                    )));
                }
            },
            None => return Err(ResponseError::new(format!("Missing user group in {}", s))),
        };
        let description = match split.get(4) {
            Some(description) => description,
            None => {
                return Err(ResponseError::new(format!(
                    "Missing user description in {}",
                    s
                )));
            }
        };
        let home = match split.get(5) {
            Some(home) => home,
            None => return Err(ResponseError::new(format!("Missing user home in {}", s))),
        };
        let login = match split.get(6) {
            Some(login) => login,
            None => return Err(ResponseError::new(format!("Missing user login in {}", s))),
        };

        Ok(User {
            name: name.to_string(),
            id,
            group,
            description: description.to_string(),
            home: home.to_string(),
            login: login.to_string(),
        })
    }
}

impl FromStr for Group {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(":").collect();

        let name = match split.first() {
            Some(name) => name,
            None => return Err(ResponseError::new(format!("Missing user name in {}", s))),
        };
        let id = match split.get(2) {
            Some(id) => match u32::from_str(id) {
                Ok(id) => id,
                Err(e) => {
                    return Err(ResponseError::new(format!(
                        "Could not convert user id {} to u32: {}",
                        id, e
                    )));
                }
            },
            None => return Err(ResponseError::new(format!("Missing user id in {}", s))),
        };
        let members: Vec<String> = match split.get(3) {
            Some(members) => {
                if members.is_empty() {
                    vec![]
                } else {
                    members.split(",").map(|s| s.to_string()).collect()
                }
            }
            None => return Err(ResponseError::new(format!("Missing user group in {}", s))),
        };

        Ok(Group {
            name: name.to_string(),
            id,
            members,
        })
    }
}

pub fn get_users(prefix: Option<PathBuf>) -> Result<Vec<User>, ResponseError> {
    let path = prefix
        .unwrap_or(Path::new("/").to_path_buf())
        .join("etc")
        .join("passwd");

    let file_content = match read_to_string(&path) {
        Ok(file_content) => file_content,
        Err(e) => {
            return Err(ResponseError::new(e.to_string()));
        }
    };

    file_content
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(User::from_str)
        .collect::<Result<Vec<User>, ResponseError>>()
}

pub fn get_groups(prefix: Option<PathBuf>) -> Result<Vec<Group>, ResponseError> {
    let path = prefix
        .unwrap_or(Path::new("/").to_path_buf())
        .join("etc")
        .join("group");

    let file_content = match read_to_string(&path) {
        Ok(file_content) => file_content,
        Err(e) => {
            return Err(ResponseError::new(e.to_string()));
        }
    };

    file_content
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(Group::from_str)
        .collect::<Result<Vec<Group>, ResponseError>>()
}
