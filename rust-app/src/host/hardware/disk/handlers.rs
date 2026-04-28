use std::path::Path;

use actix_web::{Responder, get, web};

use crate::{
    common::{
        btrfs::filesystem::show,
        file::{ReadFolderOptions, read_folder},
        response::{ResponseResult, json_response},
    },
    host::hardware::disk::models::Disk,
};

use super::models::Usage;

#[get("/")]
async fn endpoint() -> ResponseResult<impl Responder> {
    let path = "/dev/mapper";
    read_folder(path, &ReadFolderOptions { metadata: None })
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| item.name)
                .filter(|name| {
                    name.strip_prefix("disk")
                        .map(|rest| rest.chars().all(|c| c.is_ascii_digit()))
                        .unwrap_or(false)
                })
                .map(|name| Disk { id: name })
                .collect::<Vec<Disk>>()
        })
        .map(json_response)
}

#[get("/usage")]
pub async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let disk = path.into_inner();

    show(Path::new("/dev/mapper").join(&disk))
        .await
        .map(|show| Usage {
            total: show.total,
            used: show.used,
        })
        .map(json_response)
}
