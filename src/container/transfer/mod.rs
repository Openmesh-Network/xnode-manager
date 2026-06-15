use actix_web::{dev::HttpServiceFactory, web};

pub mod backup;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/transfer").service(backup::service())
}
