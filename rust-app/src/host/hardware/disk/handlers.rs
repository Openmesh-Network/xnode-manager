use std::path::Path;

use actix_web::{Responder, get, web};
use futures::future::join_all;

use crate::common::{
    btrfs::filesystem::show,
    file::{ReadFolderOptions, read_folder},
    response::{ResponseResult, json_response},
};

use super::models::{Disk, DiskOptions, Usage};

#[get("/")]
async fn endpoint(options: web::Query<DiskOptions>) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    let path = "/dev/mapper";
    let disks = read_folder(path, &ReadFolderOptions { metadata: None })
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
                .collect::<Vec<String>>()
        })?;

    let mut items = vec![];

    for disk in disks {
        let get = async move {
            let mut disk_usage = None;
            if options.usage.unwrap_or(false) {
                disk_usage = usage(&disk).await.ok();
            }

            Disk {
                id: disk,
                usage: disk_usage,
            }
        };

        items.push(get);
    }

    Ok(json_response(join_all(items).await))
}

#[get("/usage")]
pub async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let disk = path.into_inner();

    usage(&disk).await.map(json_response)
}

async fn usage(disk: impl AsRef<str>) -> ResponseResult<Usage> {
    let disk = disk.as_ref();

    show(Path::new("/dev/mapper").join(disk))
        .await
        .map(|show| Usage {
            total: show.total,
            used: show.used,
        })
}
