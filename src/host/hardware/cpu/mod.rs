use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/cpu").configure(|cfg| {
        cfg.service(handlers::cpu_endpoint);
        cfg.service(web::scope("/{cpu}").configure(|cfg| {
            cfg.service(handlers::info_endpoint);
            cfg.service(handlers::usage_endpoint);
        }));
    })
}
