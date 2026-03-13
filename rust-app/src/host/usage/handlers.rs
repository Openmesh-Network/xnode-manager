use actix_web::{HttpResponse, Responder, get, web};
use sysinfo::{Disks, Networks};

use crate::common::error::ResponseError;

use super::models::{AppData, CpuUsage, DiskUsage, GpuUsage, MemoryUsage, NetworkUsage};

#[get("/cpu")]
async fn cpu(data: web::Data<AppData>) -> impl Responder {
    let mut sys = match data.system.lock() {
        Ok(system) => system,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting system lock: {e}"
            )));
        }
    };

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
async fn memory(data: web::Data<AppData>) -> impl Responder {
    let mut sys = match data.system.lock() {
        Ok(system) => system,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ResponseError::new(format!(
                "Error getting system lock: {e}"
            )));
        }
    };

    sys.refresh_memory();
    let response: MemoryUsage = MemoryUsage {
        used: sys.used_memory(),
        total: sys.total_memory(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/disk")]
async fn disk() -> impl Responder {
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

#[get("/network")]
async fn network() -> impl Responder {
    let networks = Networks::new_with_refreshed_list();
    let response: Vec<NetworkUsage> = networks
        .list()
        .iter()
        .map(|(interface, data)| NetworkUsage {
            name: interface.clone(),
            mac: data.mac_address().to_string(),
            addresses: data.ip_networks().iter().map(|ip| ip.to_string()).collect(),
            received: data.total_received(),
            transmitted: data.total_transmitted(),
        })
        .collect();
    HttpResponse::Ok().json(response)
}

#[get("/gpu")]
async fn gpu(data: web::Data<AppData>) -> impl Responder {
    let mut response: Vec<GpuUsage> = vec![];

    // nvidia
    if let Ok(nvml) = &data.nvml {
        let nvml = match nvml.lock() {
            Ok(nvml) => nvml,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(ResponseError::new(format!("Error getting nvml lock: {e}")));
            }
        };

        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                if let Ok(device) = nvml.device_by_index(i)
                    && let Ok(id) = device.uuid()
                    && let Ok(name) = device.name()
                    && let Ok(utilization_rates) = device.utilization_rates()
                    && let Ok(memory_info) = device.memory_info()
                {
                    response.push(GpuUsage::Nvidia {
                        id,
                        name,
                        compute: utilization_rates.gpu as f32,
                        memory: MemoryUsage {
                            used: memory_info.used,
                            total: memory_info.total,
                        },
                        power: device.power_usage().ok(),
                    });
                }
            }
        }
    }

    HttpResponse::Ok().json(response)
}
