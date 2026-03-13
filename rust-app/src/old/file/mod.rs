use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/file".to_string()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::read_file);
    cfg.service(handlers::write_file);
    cfg.service(handlers::remove_file);
    cfg.service(handlers::read_directory);
    cfg.service(handlers::create_directory);
    cfg.service(handlers::remove_directory);
    cfg.service(handlers::get_permissions);
    cfg.service(handlers::set_permissions);
}
