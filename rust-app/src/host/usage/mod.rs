use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/usage".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope())
        .app_data(web::Data::new(models::AppData::default()))
        .configure(|cfg| {
            cfg.service(handlers::cpu_endpoint);
            cfg.service(handlers::memory_endpoint);
            cfg.service(handlers::disk_endpoint);
            cfg.service(handlers::network_endpoint);
            cfg.service(handlers::gpu_endpoint);
        })
}
