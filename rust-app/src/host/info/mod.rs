use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn scope() -> String {
    "/info".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope()).configure(|cfg| {
        cfg.service(handlers::flake);
        cfg.service(handlers::eval);
        cfg.service(handlers::users);
        cfg.service(handlers::groups);
    })
}
