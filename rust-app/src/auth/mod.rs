use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;
pub mod utils;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::login)
        .service(handlers::logout)
        .service(handlers::scopes);
}
