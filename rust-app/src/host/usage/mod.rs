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
            cfg.service(handlers::cpu);
            cfg.service(handlers::memory);
            cfg.service(handlers::disk);
            cfg.service(handlers::network);
            cfg.service(handlers::gpu);
        })
}
