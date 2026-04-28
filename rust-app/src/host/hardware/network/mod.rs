use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/network")
        .configure(|cfg| {
            cfg.service(handlers::endpoint);
        })
        .service(web::scope("/{network}").configure(|cfg| {
            cfg.service(handlers::info_endpoint);
            cfg.service(handlers::usage_endpoint);
        }))
}
