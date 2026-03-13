use actix_web::{dev::HttpServiceFactory, web};

pub mod file;
pub mod info;
pub mod usage;

pub fn scope() -> String {
    "/host".to_string()
}

pub fn service() -> impl HttpServiceFactory {
    web::scope(&scope())
        .service(file::service())
        .service(info::service())
        .service(usage::service())
}
