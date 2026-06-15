use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/backup").configure(|cfg| {
        cfg.service(handlers::backup_endpoint);
        cfg.service(web::scope("/{backup}").configure(|cfg| {
            cfg.service(handlers::create_endpoint);
            cfg.service(handlers::restore_endpoint);
            cfg.service(handlers::remove_endpoint);
        }));
    })
}
