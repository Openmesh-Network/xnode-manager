use actix_web::{dev::HttpServiceFactory, web};

pub mod config;
pub mod file;
pub mod handlers;
pub mod hardware;
pub mod info;
pub mod permission;
pub mod power;
pub mod process;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/host")
        .service(config::service())
        .service(file::service())
        .service(hardware::service())
        .service(info::service())
        .service(permission::service())
        .service(power::service())
        .service(process::service())
}
