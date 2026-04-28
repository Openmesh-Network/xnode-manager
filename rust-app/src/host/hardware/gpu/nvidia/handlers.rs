use actix_web::{Responder, get, web};

use crate::{
    common::response::{ResponseError, ResponseResult, json_response},
    host::hardware::gpu::nvidia::models::Gpu,
};

use super::models::{AppData, Info, MemoryUsage, Usage};

#[get("/")]
async fn endpoint(data: web::Data<AppData>) -> ResponseResult<impl Responder> {
    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    let count = nvml
        .device_count()
        .map_err(|e| ResponseError::new(format!("Could not get nvml count: {e}")))?;

    let mut devices = vec![];

    for i in 0..count {
        if let Ok(device) = nvml.device_by_index(i)
            && let Ok(id) = device.uuid()
        {
            devices.push(Gpu { id });
        }
    }

    Ok(json_response(devices))
}

#[get("/info")]
async fn info_endpoint(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> ResponseResult<impl Responder> {
    let uuid = path.into_inner();

    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    let device = nvml
        .device_by_uuid(uuid.clone())
        .map_err(|e| ResponseError::new(format!("Could not get device {uuid}: {e}")))?;

    let name = device
        .name()
        .map_err(|e| ResponseError::new(format!("Could not get name of device {uuid}: {e}")))?;

    Ok(json_response(Info { name }))
}

#[get("/usage")]
async fn usage_endpoint(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> ResponseResult<impl Responder> {
    let uuid = path.into_inner();

    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    let device = nvml
        .device_by_uuid(uuid.clone())
        .map_err(|e| ResponseError::new(format!("Could not get device {uuid}: {e}")))?;

    let utilization_rates = device.utilization_rates().map_err(|e| {
        ResponseError::new(format!(
            "Could not get utilization_rates of device {uuid}: {e}"
        ))
    })?;

    let memory_info = device.memory_info().map_err(|e| {
        ResponseError::new(format!("Could not get memory_info of device {uuid}: {e}"))
    })?;

    Ok(json_response(Usage {
        compute: utilization_rates.gpu,
        memory: MemoryUsage {
            total: memory_info.total,
            available: memory_info.free,
        },
        power: device.power_usage().ok(),
    }))
}
