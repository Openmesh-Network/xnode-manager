use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::cpu);
    cfg.service(handlers::memory);
    cfg.service(handlers::disk);
}
