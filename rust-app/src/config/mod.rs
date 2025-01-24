use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::containers);
    cfg.service(handlers::container);
    cfg.service(handlers::change);
}
