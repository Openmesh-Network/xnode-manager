use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/process".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::list_endpoint);
        cfg.service(handlers::logs_endpoint);
        cfg.service(handlers::usage_endpoint);
        cfg.service(handlers::start_endpoint);
        cfg.service(handlers::stop_endpoint);
        cfg.service(handlers::restart_endpoint);
    })
}
