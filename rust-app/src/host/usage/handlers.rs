use actix_web::{Responder, get, web};
use nvml_wrapper::Nvml;
use sysinfo::{Disks, Networks, System};

use crate::common::{error::ResponseError, response::wrap_json_response};

use super::models::{AppData, CpuUsage, DiskUsage, GpuUsage, MemoryUsage, NetworkUsage};

#[get("/cpu")]
async fn cpu_endpoint(data: web::Data<AppData>) -> impl Responder {
    let mut system = data.system.lock().await;
    wrap_json_response(cpu(&mut system).await)
}

#[get("/memory")]
async fn memory_endpoint(data: web::Data<AppData>) -> impl Responder {
    let mut system = data.system.lock().await;
    wrap_json_response(memory(&mut system).await)
}

#[get("/disk")]
async fn disk_endpoint() -> impl Responder {
    wrap_json_response(disk().await)
}

#[get("/network")]
async fn network_endpoint() -> impl Responder {
    wrap_json_response(network().await)
}

#[get("/gpu")]
async fn gpu_endpoint(data: web::Data<AppData>) -> impl Responder {
    let nvml = data.nvml.lock().await;
    wrap_json_response(gpu(nvml.as_ref().ok()).await)
}

pub async fn cpu(system: &mut System) -> Result<Vec<CpuUsage>, ResponseError> {
    system.refresh_cpu_all();
    Ok(system
        .cpus()
        .iter()
        .map(|cpu| CpuUsage {
            name: cpu.name().to_string(),
            used: cpu.cpu_usage(),
            frequency: cpu.frequency(),
        })
        .collect())
}

pub async fn memory(system: &mut System) -> Result<MemoryUsage, ResponseError> {
    system.refresh_memory();
    Ok(MemoryUsage {
        used: system.used_memory(),
        total: system.total_memory(),
    })
}

pub async fn disk() -> Result<Vec<DiskUsage>, ResponseError> {
    let disks = Disks::new_with_refreshed_list();
    Ok(disks
        .list()
        .iter()
        .map(|disk| DiskUsage {
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total: disk.total_space(), // TODO run `btrfs filesystem df` per mount point for more accurate usage info
            used: disk.total_space() - disk.available_space(),
            read: disk.usage().total_read_bytes,
            written: disk.usage().total_written_bytes,
        })
        .collect())
}

pub async fn network() -> Result<Vec<NetworkUsage>, ResponseError> {
    let networks = Networks::new_with_refreshed_list();
    Ok(networks
        .list()
        .iter()
        .map(|(interface, data)| NetworkUsage {
            name: interface.clone(),
            mac: data.mac_address().to_string(),
            addresses: data.ip_networks().iter().map(|ip| ip.to_string()).collect(),
            received: data.total_received(),
            transmitted: data.total_transmitted(),
        })
        .collect())
}

pub async fn gpu(nvml: Option<&Nvml>) -> Result<Vec<GpuUsage>, ResponseError> {
    let mut response: Vec<GpuUsage> = vec![];

    // nvidia
    if let Some(nvml) = nvml
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

    Ok(response)
}
