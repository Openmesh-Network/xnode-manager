use actix_web::{Responder, get, web};
use sysinfo::{Disks, Networks};

use crate::common::response::json_response;

use super::models::{AppData, CpuUsage, DiskUsage, GpuUsage, MemoryUsage, NetworkUsage};

#[get("/cpu")]
async fn cpu_endpoint(data: web::Data<AppData>) -> impl Responder {
    let mut system = data.system.lock().await;
    system.refresh_cpu_all();
    json_response(
        system
            .cpus()
            .iter()
            .map(|cpu| CpuUsage {
                name: cpu.name().to_string(),
                used: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect::<Vec<CpuUsage>>(),
    )
}

#[get("/memory")]
async fn memory_endpoint(data: web::Data<AppData>) -> impl Responder {
    let mut system = data.system.lock().await;
    system.refresh_memory();
    json_response(MemoryUsage {
        used: system.used_memory(),
        total: system.total_memory(),
    })
}

#[get("/disk")]
async fn disk_endpoint() -> impl Responder {
    let disks = Disks::new_with_refreshed_list();
    json_response(
        disks
            .list()
            .iter()
            .map(|disk| DiskUsage {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total: disk.total_space(), // TODO run `btrfs filesystem df` per mount point for more accurate usage info
                used: disk.total_space() - disk.available_space(),
                read: disk.usage().total_read_bytes,
                written: disk.usage().total_written_bytes,
            })
            .collect::<Vec<DiskUsage>>(),
    )
}

#[get("/network")]
async fn network_endpoint() -> impl Responder {
    let networks = Networks::new_with_refreshed_list();
    json_response(
        networks
            .list()
            .iter()
            .map(|(interface, data)| NetworkUsage {
                name: interface.clone(),
                mac: data.mac_address().to_string(),
                addresses: data.ip_networks().iter().map(|ip| ip.to_string()).collect(),
                received: data.total_received(),
                transmitted: data.total_transmitted(),
            })
            .collect::<Vec<NetworkUsage>>(),
    )
}

#[get("/gpu")]
async fn gpu_endpoint(data: web::Data<AppData>) -> impl Responder {
    let nvml = data.nvml.lock().await;
    let mut response: Vec<GpuUsage> = vec![];

    // nvidia
    if let Ok(nvml) = nvml.as_ref()
        && let Ok(count) = nvml.device_count()
    {
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

    json_response(response)
}
