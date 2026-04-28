use std::str::FromStr;

use actix_web::{Responder, get};

use crate::common::{
    file::read_file,
    response::{ResponseError, ResponseResult, json_response},
    string::escaped_utf8_from_bytes,
};

use super::models::Usage;

#[get("/usage")]
pub async fn usage_endpoint() -> ResponseResult<impl Responder> {
    let path = "/proc/meminfo";

    let file_content = read_file(&path).await.map(escaped_utf8_from_bytes)?;

    let usage = Usage::from_str(&file_content)?;

    Ok(json_response(usage))
}

impl FromStr for Usage {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mem_total = None;
        let mut mem_available = None;

        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if let Some(value) = parts.first() {
                    mem_total = Some(value.parse::<u64>().map_err(|e| {
                        ResponseError::new(format!("Could not parse MemTotal in {s} to u64: {e}"))
                    })?)
                }
            }

            if let Some(rest) = line.strip_prefix("MemAvailable:") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if let Some(value) = parts.first() {
                    mem_available = Some(value.parse::<u64>().map_err(|e| {
                        ResponseError::new(format!(
                            "Could not parse MemAvailable in {s} to u64: {e}"
                        ))
                    })?)
                }
            }
        }

        let total =
            mem_total.ok_or_else(|| ResponseError::new(format!("Missing MemTotal in {s}")))?;

        let available = mem_available
            .ok_or_else(|| ResponseError::new(format!("Missing MemAvailable in {s}")))?;

        // convert kB to bytes
        let total_bytes = total * 1024;
        let available_bytes = available * 1024;

        Ok(Usage {
            total: total_bytes,
            available: available_bytes,
        })
    }
}
