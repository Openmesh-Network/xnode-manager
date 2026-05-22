use actix_web::{Responder, post};
use tokio::process::Command;

use crate::common::{
    command::execute_command_simple,
    env::systemd,
    response::{ResponseError, ResponseResult, raw_response},
};

#[post("/off")]
async fn off_endpoint() -> ResponseResult<impl Responder> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.arg("poweroff");
    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| raw_response(()))
        .map_err(|e| ResponseError::new(format!("Could not shutdown: {e}")))
}

#[post("/reboot")]
async fn reboot_endpoint() -> ResponseResult<impl Responder> {
    let mut command = Command::new(format!("{}systemctl", systemd()));
    command.arg("reboot");
    execute_command_simple(command, None::<Vec<u8>>)
        .await
        .map(|_output| raw_response(()))
        .map_err(|e| ResponseError::new(format!("Could not reboot: {e}")))
}
