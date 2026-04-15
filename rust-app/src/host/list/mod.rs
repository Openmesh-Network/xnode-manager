use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/list".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::process_endpoint);
        cfg.service(handlers::container_endpoint);
    })
}
