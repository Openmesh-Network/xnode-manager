use std::str::FromStr;

use actix_web::{Responder, get, web};
use futures::future::join_all;

use crate::common::{
    file::{ReadFolderOptions, read_file, read_folder},
    response::{ResponseError, ResponseResult, json_response},
    string::escaped_utf8_from_bytes,
};

use super::models::{Cpu, CpuOptions, Info, Usage};

#[get("/")]
async fn endpoint(options: web::Query<CpuOptions>) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    let path = "/sys/devices/system/cpu";
    let cpus = read_folder(path, &ReadFolderOptions { metadata: None })
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| item.name)
                .filter(|name| {
                    name.strip_prefix("cpu")
                        .map(|rest| rest.chars().all(|c| c.is_ascii_digit()))
                        .unwrap_or(false)
                })
                .map(|name| name.replace("cpu", ""))
                .collect::<Vec<String>>()
        })?;

    let mut items = vec![];

    for cpu in cpus {
        let get = async move {
            let mut cpu_usage = None;
            if options.usage.unwrap_or(false) {
                cpu_usage = usage(&cpu).await.ok();
            }

            Cpu {
                id: cpu,
                usage: cpu_usage,
            }
        };

        items.push(get);
    }

    Ok(json_response(join_all(items).await))
}

#[get("/info")]
async fn info_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let cpu = path.into_inner();

    let path = "/proc/cpuinfo";

    let file_content = read_file(&path).await.map(escaped_utf8_from_bytes)?;

    file_content
        .split("\n\n")
        .find(|block| {
            block.lines().any(|line| {
                if let Some((key, value)) = line.split_once(':')
                    && key.trim() == "processor"
                {
                    value.trim() == cpu
                } else {
                    false
                }
            })
        })
        .ok_or_else(|| ResponseError::new(format!("Cpu {cpu} not found in {path}.")))
        .and_then(Info::from_str)
        .map(json_response)
}

#[get("/usage")]
async fn usage_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let cpu = path.into_inner();
    usage(cpu).await.map(json_response)
}

async fn usage(cpu: impl AsRef<str>) -> ResponseResult<Usage> {
    let cpu = cpu.as_ref();

    let path = "/proc/stat";

    let file_content = read_file(&path).await.map(escaped_utf8_from_bytes)?;

    file_content
        .lines()
        .find(|line| line.starts_with(&format!("cpu{cpu} ")))
        .ok_or_else(|| ResponseError::new(format!("Cpu {cpu} not found in {path}.")))
        .and_then(Usage::from_str)
}

impl FromStr for Info {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut name = None;
        let mut flags = None;

        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("model name")
                && let Some((_, value)) = rest.split_once(':')
            {
                name = Some(value.trim().to_string());
            }

            if let Some(rest) = line.strip_prefix("flags")
                && let Some((_, value)) = rest.split_once(':')
            {
                flags = Some(
                    value
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );
            }
        }

        let name = name.ok_or_else(|| ResponseError::new(format!("Missing model name in {s}")))?;
        let flags = flags.ok_or_else(|| ResponseError::new(format!("Missing flags in {s}")))?;

        Ok(Info { name, flags })
    }
}

impl FromStr for Usage {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace().skip(1);

        macro_rules! next_u64 {
            () => {
                iter.next()
                    .ok_or_else(|| ResponseError::new(format!("Missing CPU usage field in {s}")))?
                    .parse::<u64>()
                    .map_err(|e| {
                        ResponseError::new(format!("Could not parse CPU usage value in {s}: {e}"))
                    })?
            };
        }

        Ok(Usage {
            user: next_u64!(),
            nice: next_u64!(),
            system: next_u64!(),
            idle: next_u64!(),
            iowait: next_u64!(),
            irq: next_u64!(),
            softirq: next_u64!(),
            steal: next_u64!(),
            guest: next_u64!(),
            guest_nice: next_u64!(),
        })
    }
}
