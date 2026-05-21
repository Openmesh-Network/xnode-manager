use actix_web::{dev::HttpServiceFactory, web};

pub mod container;
pub mod handlers;
pub mod models;
pub mod virtual_machine;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/permission")
        .service(container::service())
        .service(virtual_machine::service())
}
