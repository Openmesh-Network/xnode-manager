use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/info").configure(|cfg| {
        cfg.service(handlers::flake_metadata_endpoint);
        cfg.service(handlers::eval_endpoint);
        cfg.service(handlers::users_users_endpoint);
        cfg.service(handlers::users_groups_endpoint);
    })
}
