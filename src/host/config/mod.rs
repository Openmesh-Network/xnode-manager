use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;
pub mod secret;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/config")
        .service(secret::service())
        .configure(|cfg| {
            cfg.service(handlers::get_endpoint);
            cfg.service(handlers::set_endpoint);
            cfg.service(handlers::version_endpoint);
            cfg.service(handlers::update_endpoint);
            cfg.service(handlers::build_endpoint);
            cfg.service(handlers::apply_endpoint);
        })
}
