use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder};
use sysinfo::Disks;

use super::models::{CpuUsage, DiskUsage, MemoryUsage};
use crate::{
    auth::{models::Scope, utils::has_permission},
    usage::models::AppData,
    utils::error::ResponseError,
};

#[get("/cpu")]
async fn cpu(user: Identity, data: web::Data<AppData>) -> impl Responder {
    if !has_permission(user, Scope::Usage) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut sys;
    match data.system.lock() {
        Ok(system) => {
            sys = system;
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting system info: {}",
                e
            )))
        }
    }

    sys.refresh_cpu_all();
    let response: Vec<CpuUsage> = sys
        .cpus()
        .iter()
        .map(|cpu| CpuUsage {
            name: cpu.name().to_string(),
            used: cpu.cpu_usage(),
            frequency: cpu.frequency(),
        })
        .collect();
    HttpResponse::Ok().json(response)
}

#[get("/memory")]
async fn memory(user: Identity, data: web::Data<AppData>) -> impl Responder {
    if !has_permission(user, Scope::Usage) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut sys;
    match data.system.lock() {
        Ok(system) => {
            sys = system;
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting system info: {}",
                e
            )))
        }
    }

    sys.refresh_memory();
    let response: MemoryUsage = MemoryUsage {
        used: sys.used_memory(),
        total: sys.total_memory(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/disk")]
async fn disk(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Usage) {
        return HttpResponse::Unauthorized().finish();
    }

    let disks = Disks::new_with_refreshed_list();
    let response: Vec<DiskUsage> = disks
        .list()
        .iter()
        .map(|disk| DiskUsage {
            mount_point: disk
                .mount_point()
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Non-UTF8 mount point".to_string()),
            total: disk.total_space(),
            used: disk.total_space() - disk.available_space(),
        })
        .collect();
    HttpResponse::Ok().json(response)
}
