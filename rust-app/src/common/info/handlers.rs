use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use tokio::fs::read_to_string;

use crate::common::error::ResponseError;

use super::models::{Group, User};

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

pub async fn get_users(prefix: Option<PathBuf>) -> Result<Vec<User>, ResponseError> {
    let path = prefix
        .unwrap_or(Path::new("/").to_path_buf())
        .join("etc")
        .join("passwd");

    let file_content = match read_to_string(&path).await {
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

pub async fn get_groups(prefix: Option<PathBuf>) -> Result<Vec<Group>, ResponseError> {
    let path = prefix
        .unwrap_or(Path::new("/").to_path_buf())
        .join("etc")
        .join("group");

    let file_content = match read_to_string(&path).await {
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
