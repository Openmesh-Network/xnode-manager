use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::common::{error::ResponseError, file::read_file, string::escaped_utf8_from_bytes};

use super::models::{Group, User};

impl FromStr for User {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(":").collect();

        let name = split
            .first()
            .ok_or(ResponseError::new(format!("Missing user name in {s}")))?;

        let id = split
            .get(2)
            .ok_or(ResponseError::new(format!("Missing user id in {s}")))?;
        let id = u32::from_str(id).map_err(|e| {
            ResponseError::new(format!("Could not convert user id {id} to u32: {e}"))
        })?;

        let group = split
            .get(3)
            .ok_or(ResponseError::new(format!("Missing user group in {s}")))?;
        let group = u32::from_str(group).map_err(|e| {
            ResponseError::new(format!("Could not convert user group {group} to u32: {e}"))
        })?;

        let description = split.get(4).ok_or(ResponseError::new(format!(
            "Missing user description in {s}"
        )))?;

        let home = split
            .get(5)
            .ok_or(ResponseError::new(format!("Missing user home in {s}")))?;

        let login = split
            .get(6)
            .ok_or(ResponseError::new(format!("Missing user login in {s}")))?;

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

        let name = split
            .first()
            .ok_or(ResponseError::new(format!("Missing user name in {s}")))?;

        let id = split
            .get(2)
            .ok_or(ResponseError::new(format!("Missing user id in {s}")))?;
        let id = u32::from_str(id).map_err(|e| {
            ResponseError::new(format!("Could not convert user id {id} to u32: {e}"))
        })?;

        let members: Vec<String> = split
            .get(3)
            .map(|members| {
                if members.is_empty() {
                    vec![]
                } else {
                    members.split(",").map(|s| s.to_string()).collect()
                }
            })
            .ok_or(ResponseError::new(format!("Missing user group in {s}")))?;

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

    let file_content = read_file(&path).await.map(escaped_utf8_from_bytes)?;

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

    let file_content = read_file(&path).await.map(escaped_utf8_from_bytes)?;

    file_content
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(Group::from_str)
        .collect::<Result<Vec<Group>, ResponseError>>()
}
