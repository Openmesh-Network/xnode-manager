use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/config".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::get_endpoint);
        cfg.service(handlers::set_endpoint);
        cfg.service(handlers::version_endpoint);
        cfg.service(handlers::update_endpoint);
        cfg.service(handlers::apply_endpoint);
    })
}
