use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/memory").configure(|cfg| {
        cfg.service(handlers::usage_endpoint);
    })
}
