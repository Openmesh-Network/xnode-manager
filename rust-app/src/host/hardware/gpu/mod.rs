use actix_web::{dev::HttpServiceFactory, web};

pub mod nvidia;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/gpu").service(nvidia::service())
}
