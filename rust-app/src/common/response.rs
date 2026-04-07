use std::{fmt::Display, io::ErrorKind, path::Path};

use actix_web::{HttpResponse, Responder, body::MessageBody};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseError {
    pub error: String,
    pub typed_error: Option<TypedResponseError>,
}

impl ResponseError {
    pub fn new(error: impl Display) -> Self {
        let error = error.to_string();
        log::warn!("Response error: {error}");

        Self {
            error,
            typed_error: None,
        }
    }

    pub fn typed(typed_error: TypedResponseError) -> Self {
        let error = typed_error.to_string();
        log::warn!("Response error: {error}");

        Self {
            error,
            typed_error: Some(typed_error),
        }
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl actix_web::ResponseError for ResponseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(self)
    }
}

pub type ResponseResult<T> = Result<T, ResponseError>;

pub fn json_response<T: serde::Serialize>(body: T) -> impl Responder {
    HttpResponse::Ok().json(body)
}

pub fn raw_response<T: MessageBody + 'static>(body: T) -> impl Responder {
    HttpResponse::Ok().body(body)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypedResponseError {
    PathNotFound { path: String },
}

impl Display for TypedResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypedResponseError::PathNotFound { path } => format!("Could not find path {path}."),
            }
        )
    }
}

impl TypedResponseError {
    pub fn from_io(error: &std::io::Error, path: impl AsRef<Path>) -> Option<Self> {
        match error.kind() {
            ErrorKind::NotFound => Some(TypedResponseError::PathNotFound {
                path: path.as_ref().to_string_lossy().to_string(),
            }),
            _ => None,
        }
    }
}
