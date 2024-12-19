use actix_identity::Identity;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use super::models::{Login, Scope};
use super::utils::get_scopes;

#[get("/scopes")]
async fn scopes(user: Identity) -> impl Responder {
    let scopes = get_scopes(user);

    let response: Vec<&Scope> = Vec::from_iter(scopes.iter());
    HttpResponse::Ok().json(response)
}

#[post("/login")]
async fn login(login: web::Json<Login>, request: HttpRequest) -> impl Responder {
    // Some kind of authentication should happen here
    // e.g. password-based, biometric, etc.
    // [...]

    // attach a verified user identity to the active session
    Identity::login(&request.extensions(), login.user.clone()).unwrap();

    HttpResponse::Ok()
}

#[post("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::Ok()
}
