use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/os".to_string()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::get);
    cfg.service(handlers::set);
    cfg.service(handlers::reboot);
}
