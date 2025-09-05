use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/info".to_string()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::flake);
    cfg.service(handlers::users);
    cfg.service(handlers::groups);
}
