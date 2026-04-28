use actix_web::{Responder, get, web};
use futures::future::join_all;
use nvml_wrapper::Nvml;

use crate::common::response::{ResponseError, ResponseResult, json_response};

use super::models::{AppData, Gpu, GpuOptions, Info, MemoryUsage, Usage};

#[get("/")]
async fn endpoint(
    data: web::Data<AppData>,
    options: web::Query<GpuOptions>,
) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    let count = nvml
        .device_count()
        .map_err(|e| ResponseError::new(format!("Could not get nvml count: {e}")))?;

    let mut items = vec![];

    for i in 0..count {
        if let Ok(device) = nvml.device_by_index(i)
            && let Ok(gpu) = device.uuid()
        {
            let get = async move {
                let mut gpu_usage = None;
                if options.usage.unwrap_or(false) {
                    gpu_usage = usage(nvml, &gpu).await.ok();
                }

                Gpu {
                    id: gpu,
                    usage: gpu_usage,
                }
            };

            items.push(get);
        }
    }

    Ok(json_response(join_all(items).await))
}

#[get("/info")]
async fn info_endpoint(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> ResponseResult<impl Responder> {
    let gpu = path.into_inner();

    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    let device = nvml
        .device_by_uuid(gpu.clone())
        .map_err(|e| ResponseError::new(format!("Could not get device {gpu}: {e}")))?;

    let name = device
        .name()
        .map_err(|e| ResponseError::new(format!("Could not get name of device {gpu}: {e}")))?;

    Ok(json_response(Info { name }))
}

#[get("/usage")]
async fn usage_endpoint(
    data: web::Data<AppData>,
    path: web::Path<String>,
) -> ResponseResult<impl Responder> {
    let gpu = path.into_inner();

    let nvml = data.nvml.lock().await;
    let nvml = nvml
        .as_ref()
        .map_err(|e| ResponseError::new(format!("Could not acquire nvml lock: {e}")))?;

    usage(nvml, gpu).await.map(json_response)
}

async fn usage(nvml: &Nvml, gpu: impl AsRef<str>) -> ResponseResult<Usage> {
    let gpu = gpu.as_ref();

    let device = nvml
        .device_by_uuid(gpu)
        .map_err(|e| ResponseError::new(format!("Could not get device {gpu}: {e}")))?;

    let utilization_rates = device.utilization_rates().map_err(|e| {
        ResponseError::new(format!(
            "Could not get utilization_rates of device {gpu}: {e}"
        ))
    })?;

    let memory_info = device.memory_info().map_err(|e| {
        ResponseError::new(format!("Could not get memory_info of device {gpu}: {e}"))
    })?;

    Ok(Usage {
        compute: utilization_rates.gpu,
        memory: MemoryUsage {
            total: memory_info.total,
            available: memory_info.free,
        },
        power: device.power_usage().ok(),
    })
}
