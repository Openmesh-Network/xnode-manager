use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/power").configure(|cfg| {
        cfg.service(handlers::off_endpoint);
        cfg.service(handlers::reboot_endpoint);
    })
}
