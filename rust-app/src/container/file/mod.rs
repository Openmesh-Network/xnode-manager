use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/file".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::metadata_endpoint);
        cfg.service(handlers::size_endpoint);
        cfg.service(handlers::move_endpoint);
        cfg.service(handlers::remove_endpoint);
        cfg.service(handlers::copy_endpoint);
        cfg.service(handlers::read_file_endpoint);
        cfg.service(handlers::write_file_endpoint);
        cfg.service(handlers::read_folder_endpoint);
        cfg.service(handlers::create_folder_endpoint);
        cfg.service(handlers::read_link_endpoint);
        cfg.service(handlers::write_link_endpoint);
        cfg.service(handlers::get_permissions_endpoint);
        cfg.service(handlers::set_permissions_endpoint);
    })
}
