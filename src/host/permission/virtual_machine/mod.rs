use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/virtual-machine/{virtual_machine}").configure(|cfg| {
        cfg.service(handlers::get_endpoint);
        cfg.service(handlers::set_endpoint);
    })
}
