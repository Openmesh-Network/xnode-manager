use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/backup").configure(|cfg| {
        cfg.service(handlers::receive_endpoint);
        cfg.service(handlers::send_endpoint);
    })
}
