use actix_identity::Identity;
use actix_web::{get, HttpResponse, Responder};
use sysinfo::{Disks, System};

use super::models::{CpuUsage, DiskUsage, MemoryUsage};
use crate::auth::{models::Scope, utils::has_permission};

#[get("/cpu")]
async fn cpu(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Read) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut sys = System::new();
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
async fn memory(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Read) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut sys = System::new();
    sys.refresh_memory();
    let response: MemoryUsage = MemoryUsage {
        used: sys.used_memory(),
        total: sys.total_memory(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/disk")]
async fn disk(user: Identity) -> impl Responder {
    if !has_permission(user, Scope::Read) {
        return HttpResponse::Unauthorized().finish();
    }

    let disks = Disks::new_with_refreshed_list();
    let response: Vec<DiskUsage> = disks
        .list()
        .iter()
        .map(|disk| DiskUsage {
            name: disk
                .name()
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or("Unnamed Disk".to_string()),
            total: disk.total_space(),
            used: disk.total_space() - disk.available_space(),
        })
        .collect();
    HttpResponse::Ok().json(response)
}
