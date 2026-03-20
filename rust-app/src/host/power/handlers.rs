use actix_web::{Responder, post};
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple, env::systemd, error::ResponseError,
    response::wrap_raw_response,
};

#[post("/off")]
async fn off_endpoint() -> impl Responder {
    wrap_raw_response(shutdown().await)
}

#[post("/reboot")]
async fn reboot_endpoint() -> impl Responder {
    wrap_raw_response(reboot().await)
}

pub async fn shutdown() -> Result<(), ResponseError> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.arg("poweroff");
    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| ResponseError::new(format!("Could not shutdown: {e}")))
}

pub async fn reboot() -> Result<(), ResponseError> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.arg("reboot");
    execute_command_simple(command)
        .await
        .map(|_output| ())
        .map_err(|e| ResponseError::new(format!("Could not reboot: {e}")))
}
