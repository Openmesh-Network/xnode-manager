use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/file".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::metadata);
        cfg.service(handlers::read_file);
        cfg.service(handlers::write_file);
        cfg.service(handlers::remove_file);
        cfg.service(handlers::copy_file);
        cfg.service(handlers::read_folder);
        cfg.service(handlers::create_folder);
        cfg.service(handlers::remove_folder);
        cfg.service(handlers::copy_folder);
        cfg.service(handlers::get_permissions);
        cfg.service(handlers::set_permissions);
    })
}
