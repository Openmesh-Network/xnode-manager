use std::path::Path;

use actix_web::{Responder, get, web};
use futures::future::join_all;

use crate::common::{
    btrfs::filesystem::show,
    file::{ReadFolderOptions, read_file, read_folder, read_link},
    response::{ResponseError, ResponseResult, json_response},
    string::escaped_utf8_from_bytes,
};

use super::models::{Disk, DiskOptions, Usage};

#[get("")]
async fn disk_endpoint(options: web::Query<DiskOptions>) -> ResponseResult<impl Responder> {
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
    let path = Path::new("/dev/mapper").join(disk);

    let storage = show(&path).await?;

    let link = read_link(&path).await?;
    let block_name = link.file_name().ok_or_else(|| {
        ResponseError::new(format!(
            "{path} doesn't link to a file.",
            path = path.display()
        ))
    })?;

    let (sectors_read, sectors_written) =
        read_file(Path::new("/sys/block").join(block_name).join("stat"))
            .await
            .map(escaped_utf8_from_bytes)
            .and_then(|content| {
                let mut fields = content.split_whitespace().skip(2);
                let sectors_read: u64 = fields
                    .next()
                    .ok_or_else(|| {
                        ResponseError::new(format!("Missing sectors_read from {content}"))
                    })?
                    .parse()
                    .map_err(|e| {
                        ResponseError::new(format!("Could not convert sectors_read to u64: {e}"))
                    })?;

                let mut fields = fields.skip(3);
                let sectors_written: u64 = fields
                    .next()
                    .ok_or_else(|| {
                        ResponseError::new(format!("Missing sectors_written from {content}"))
                    })?
                    .parse()
                    .map_err(|e| {
                        ResponseError::new(format!("Could not convert sectors_written to u64: {e}"))
                    })?;

                Ok((sectors_read, sectors_written))
            })?;

    Ok(Usage {
        total: storage.total,
        used: storage.used,
        read: sectors_read * 512,
        written: sectors_written * 512,
    })
}
