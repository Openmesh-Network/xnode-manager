use actix_web::{HttpResponse, Responder, body::MessageBody};

use super::error::ResponseError;

pub fn wrap_json_response<T: serde::Serialize>(result: Result<T, ResponseError>) -> impl Responder {
    match result {
        Ok(body) => HttpResponse::Ok().json(body),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

pub fn wrap_raw_response<T: MessageBody + 'static>(
    result: Result<T, ResponseError>,
) -> impl Responder {
    match result {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
