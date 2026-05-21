use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/nvidia")
        .app_data(web::Data::new(models::AppData::default()))
        .configure(|cfg| {
            cfg.service(handlers::nvidia_endpoint);
            cfg.service(web::scope("/{gpu}").configure(|cfg| {
                cfg.service(handlers::info_endpoint);
                cfg.service(handlers::usage_endpoint);
            }));
        })
}
