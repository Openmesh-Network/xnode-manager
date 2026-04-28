use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/process")
        .configure(|cfg| {
            cfg.service(handlers::endpoint);
        })
        .service(web::scope("/{process}").configure(|cfg| {
            cfg.service(handlers::info_endpoint);
            cfg.service(handlers::status_endpoint);
            cfg.service(handlers::logs_endpoint);
            cfg.service(handlers::usage_endpoint);
            cfg.service(handlers::start_endpoint);
            cfg.service(handlers::stop_endpoint);
            cfg.service(handlers::restart_endpoint);
            cfg.service(handlers::reload_endpoint);
        }))
}
