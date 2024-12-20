use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::list);
    cfg.service(handlers::logs);
}