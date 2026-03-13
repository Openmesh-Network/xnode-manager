use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/request".to_string()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::request_info);
    cfg.service(handlers::command_info);
}
