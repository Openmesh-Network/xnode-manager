use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/nvidia")
        .app_data(web::Data::new(models::AppData::default()))
        .configure(|cfg| {
            cfg.service(handlers::endpoint);
        })
        .service(web::scope("/{uuid}").configure(|cfg| {
            cfg.service(handlers::info_endpoint);
            cfg.service(handlers::usage_endpoint);
        }))
}
