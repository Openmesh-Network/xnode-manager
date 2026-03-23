use actix_web::{dev::HttpServiceFactory, web};

pub mod config;
pub mod file;
pub mod info;
pub mod permission;
pub mod power;
pub mod process;
pub mod usage;

pub fn scope() -> String {
    "/host".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope())
        .service(config::service())
        .service(file::service())
        .service(info::service())
        .service(permission::service())
        .service(power::service())
        .service(process::service())
        .service(usage::service())
}
